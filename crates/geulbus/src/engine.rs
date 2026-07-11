//! `org.freedesktop.IBus.Engine` 구현. geulbus-core 의 조합 엔진을 감싼다.
//!
//! 키 이벤트(method)를 받아 조합하고, 결과를 CommitText / UpdatePreeditText
//! (signal)로 데몬에 돌려준다. 참고: `research/03-ibus-zbus.md` §2,§4.

use std::collections::HashMap;

use geulbus_core::expr::{Ctx, Expr, Value as ExprValue};
use geulbus_core::{us_qwerty_ascii, Config, Engine as Core};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;
use zbus::{fdo, interface};

use crate::ibus_lookup_table::{make_lookup_table, PAGE_SIZE};
use crate::ibus_property::{make_input_mode_property, make_prop_list};
use crate::ibus_text::{make_ibus_text, make_preedit_text};
use crate::settings::Settings;

// 수식어/키 마스크 (research/03 §4, 실측).
const RELEASE_MASK: u32 = 1 << 30;
const SHIFT_MASK: u32 = 1 << 0;
const LOCK_MASK: u32 = 1 << 1; // Caps Lock
const CONTROL_MASK: u32 = 1 << 2;
const MOD1_MASK: u32 = 1 << 3; // Alt
const MOD4_MASK: u32 = 1 << 6; // Super/Win 키가 실제로 실려 오는 비트(표준 XKB 매핑).
const SUPER_MASK: u32 = 1 << 26; // ibus 가상 Super 마스크(설정에 따라 안 실릴 때가 많음).
const META_MASK: u32 = 1 << 28;
// 단축키 수식어: Ctrl/Alt/Super(Mod4 + 가상)/Meta. 이 비트가 있으면 텍스트 입력이
// 아니라 단축키 코드이므로 조합을 확정하고 응용/컴포지터로 통과시킨다. Super 는 가상
// SUPER_MASK 가 잘 안 실려서 Mod4 도 함께 본다(안 그러면 Super+1 등이 입력으로 새어
// geulbus 이 먹어 버린다).
const SPECIAL_MODS: u32 = CONTROL_MASK | MOD1_MASK | MOD4_MASK | SUPER_MASK | META_MASK;

// 키심(keysym).
const KEY_BACKSPACE: u32 = 0xff08;
const KEY_HANGUL: u32 = 0xff31;
const KEY_RETURN: u32 = 0xff0d;
const KEY_KP_ENTER: u32 = 0xff8d;
const KEY_ESCAPE: u32 = 0xff1b;
const KEY_LEFT: u32 = 0xff51;
const KEY_UP: u32 = 0xff52;
const KEY_RIGHT: u32 = 0xff53;
const KEY_DOWN: u32 = 0xff54;
const KEY_PAGE_UP: u32 = 0xff55;
const KEY_PAGE_DOWN: u32 = 0xff56;

/// `GEULBUS_DEBUG_KEYS` 환경변수가 켜져 있으면 키 이벤트를 stderr 로 로깅한다.
fn debug_keys_enabled() -> bool {
    matches!(
        std::env::var("GEULBUS_DEBUG_KEYS").ok().as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

/// 정수를 유니코드 아래첨자 숫자(U+2080..U+2089)로 만든다. 패널 심볼의 항목 번호용.
fn subscript_digits(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|c| char::from_u32(0x2080 + (c as u32 - '0' as u32)).unwrap_or(c))
        .collect()
}

/// 수식어 키 자체(Shift/Ctrl/Caps/Meta/Alt/Super/Hyper, ISO_Level shifts, Mode_switch)인가.
/// 이런 키는 텍스트가 아니므로 조합에 영향을 주지 않고 그대로 통과시켜야 한다.
fn is_modifier_keysym(keyval: u32) -> bool {
    (0xffe1..=0xffee).contains(&keyval) // Shift_L..Hyper_R
        || (0xfe01..=0xfe0f).contains(&keyval) // ISO_Lock, ISO_LevelN_Shift 등
        || keyval == 0xff7e // Mode_switch (AltGr 류)
}

/// 날개셋 ShortcutTable 의 가상키(VK_*) 이름 → X11/ibus 키심.
fn vk_to_keysyms(vk: &str) -> &'static [u32] {
    match vk {
        "VK_HANGUL" => &[0xff31],  // Hangul (한/영)
        "VK_HANJA" => &[0xff34],   // Hangul_Hanja (한자)
        "VK_CAPITAL" => &[0xffe5], // Caps_Lock
        "VK_SPACE" => &[0x20],
        "VK_RMENU" => &[0xffea],    // Alt_R (오른쪽 Alt, 한/영 대용)
        "VK_RCONTROL" => &[0xffe4], // Control_R (한자 대용)
        _ => &[],
    }
}

/// 키 분류(순수 함수 결과). 라우팅 로직을 D-Bus 비동기와 분리해 단위 테스트한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyClass {
    Release,
    ImeSwitch,
    /// ShortcutTable `usage="KEYCHAR"` 단축글쇠(예: 한자 키 → `C0|0x82` 후보 변환).
    Keychar,
    Modifier,
    ShortcutCombo,
    Backspace,
    Printable(u8),
    FunctionKey,
}

/// 후보창(변환 모드) 상태: 후보 전체와 절대 커서. 페이지는 커서에서 파생된다.
struct CandidateState {
    /// (확정할 문자열, 표시용 덧말 — 한자는 훈음, 특수문자는 빈 문자열).
    items: Vec<(String, String)>,
    /// 변환 대상이 된 조합 글자(후보창 보조 텍스트로 표시).
    source: String,
    /// 전체 목록 기준 커서 위치(< items.len()).
    cursor: usize,
}

impl CandidateState {
    fn page(&self) -> usize {
        PAGE_SIZE as usize
    }
    fn page_start(&self) -> usize {
        (self.cursor / self.page()) * self.page()
    }
    fn up(&mut self) {
        self.cursor = if self.cursor == 0 {
            self.items.len() - 1
        } else {
            self.cursor - 1
        };
    }
    fn down(&mut self) {
        self.cursor = if self.cursor + 1 >= self.items.len() {
            0
        } else {
            self.cursor + 1
        };
    }
    fn page_up(&mut self) {
        let p = self.page();
        self.cursor = if self.cursor >= p {
            self.cursor - p
        } else {
            // 첫 페이지에서 위로: 마지막 페이지 첫 후보로 순환.
            ((self.items.len() - 1) / p) * p
        };
    }
    fn page_down(&mut self) {
        let p = self.page();
        self.cursor = if self.cursor + p < self.items.len() {
            self.cursor + p
        } else {
            0 // 마지막 페이지에서 아래로: 첫 페이지로 순환
        };
    }
}

/// 후보창이 떠 있을 때 키 하나가 일으키는 동작(순수 상태 전이, 단위 테스트용).
#[derive(Debug, Clone, PartialEq, Eq)]
enum CandAction {
    /// 키를 소비하고 아무것도 안 함.
    Consume,
    /// 키를 통과시킴(수식어 등). 후보창 유지.
    Pass,
    /// 커서/페이지가 바뀌어 후보창을 다시 그림.
    Redraw,
    /// 이 문자열로 확정(조합 글자를 대체).
    Commit(String),
    /// 변환 취소: 후보창을 닫고 조합(preedit)은 그대로 유지.
    Cancel,
    /// 변환을 취소한 뒤 이 키를 일반 경로로 계속 처리(예: 다른 자모 입력).
    Fallthrough,
}

/// 변환 대상 글자의 후보 목록. 자음이면 특수문자 표(okpyeon mssymbol,
/// `fix_symbols` 에 따라 보정판/원본), 아니면 한자 사전(독음→한자+훈음)을 찾는다.
/// ibus-hangul 과 같은 순서(심볼 우선)다. 없으면 빈 목록.
fn build_candidates(ch: char, fix_symbols: bool) -> Vec<(String, String)> {
    let symbols = if fix_symbols {
        okpyeon::symbols_revised(ch)
    } else {
        okpyeon::symbols(ch)
    };
    if let Some(list) = symbols {
        return list
            .iter()
            .map(|&s| (s.to_string(), String::new()))
            .collect();
    }
    okpyeon::hanja(ch)
        .map(|list| {
            list.iter()
                .map(|&(h, gloss)| (h.to_string(), gloss.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// 한 입력 항목의 처리 방식.
enum Mode {
    /// 한글 조합 항목(KeyTable 에 H3| 낱자가 있는 항목). Core 가 커서 박싱.
    Hangul(Box<Core>),
    /// 로마자/직접 항목: KeyTable 로 문자만 내보내고(드보락 등), 키표가 없으면 패스스루.
    Latin { keys: HashMap<u32, Expr> },
}

impl Mode {
    fn is_hangul(&self) -> bool {
        matches!(self, Mode::Hangul(_))
    }
    /// 패널 심볼 접두(한글=가, 로마자/직접=A).
    fn symbol_prefix(&self) -> &'static str {
        if self.is_hangul() {
            "가"
        } else {
            "A"
        }
    }
}

/// IBus 엔진 인스턴스 하나. 설정의 모든 입력 항목을 담고 IME_SWITCH 로 순환 전환한다.
/// 패널 표시기는 날개셋처럼 `접두+항목번호`(예: `가0`, `A1`)로 보인다.
pub struct IBusEngine {
    entries: Vec<Mode>,
    /// 현재 활성 입력 항목 인덱스.
    current: usize,
    /// 전체 모드에서 기본으로 시작할 항목 인덱스(설정의 default).
    default_entry: usize,
    /// IME_SWITCH 키심 → 전환 대상 항목을 정하는 식. 식의 변수 `A` = 현재 항목 인덱스.
    /// 예: `!A` 는 0↔1 토글(0이면 1, 아니면 0). 설정 ShortcutTable 의 value 를 그대로 평가.
    ime_switch: HashMap<u32, Expr>,
    /// KEYCHAR 키심(한자 키 등) → 입력할 날개셋문자 식(예: `C0|0x82` 후보 변환).
    keychar: HashMap<u32, Expr>,
    /// 후보창(한자·특수문자 변환)이 떠 있으면 그 상태.
    candidates: Option<CandidateState>,
    /// 마지막으로 반영한 사용자 설정(config.ini). focus_in/enable 마다 다시 읽어 즉시 반영.
    settings: Settings,
    /// 항목 직접 지정 모드에서 쓸 항목 인덱스(settings 에서 파생, 항목 수로 클램프).
    pick_idx: usize,
    /// 클라이언트 capability 비트마스크(SetCapabilities). bit3(0x8)=surrounding-text 지원.
    caps: u32,
    /// 앱이 실제로 SetSurroundingText 를 보내준 적이 있는가. capability 비트만으로는
    /// 신뢰할 수 없어(예: 터미널이 켰다고 하고 DeleteSurroundingText 를 무시), 실제
    /// surrounding-text 가 도착한 앱에서만 BkspAttach(앞 글자 재조합)를 시도한다.
    got_surrounding: bool,
}

/// IBus capability: surrounding-text 지원 비트(IBUS_CAP_SURROUNDING_TEXT = 1<<3).
const CAP_SURROUNDING_TEXT: u32 = 1 << 3;

impl IBusEngine {
    /// 설정 파일(config.ini)을 읽어 엔진을 만든다.
    pub fn new(config: &Config) -> Self {
        Self::with_settings(config, Settings::load())
    }

    /// 명시한 설정으로 엔진을 만든다(테스트는 이걸 써서 전역 config.ini 에 의존하지 않는다).
    pub fn with_settings(config: &Config, st: Settings) -> Self {
        // 모든 입력 항목을 컴파일한다. KeyTable 에 H3| 낱자가 있으면 한글 조합 항목,
        // 아니면 로마자/직접(문자만 내보냄) 항목으로 본다.
        let mut entries = Vec::new();
        for i in 0..config.entries.len() {
            match config.compile(i) {
                Ok(layout) => {
                    let is_hangul = layout.keys.values().any(|e| e.contains_unit());
                    if is_hangul {
                        entries.push(Mode::Hangul(Box::new(Core::new(layout))));
                    } else {
                        entries.push(Mode::Latin { keys: layout.keys });
                    }
                }
                Err(_) => entries.push(Mode::Latin {
                    keys: HashMap::new(),
                }),
            }
        }
        if entries.is_empty() {
            entries.push(Mode::Latin {
                keys: HashMap::new(),
            });
        }
        let last = entries.len() - 1;

        // usage=IME_SWITCH 단축글쇠: value 식(예 "!A")을 키심에 매핑.
        // usage=KEYCHAR 단축글쇠: 날개셋문자 식(예 VK_HANJA → "C0|0x82" 후보 변환)을 매핑.
        let mut ime_switch: HashMap<u32, Expr> = HashMap::new();
        let mut keychar: HashMap<u32, Expr> = HashMap::new();
        for sc in &config.editor.shortcuts {
            let table = match sc.usage.as_str() {
                "IME_SWITCH" => &mut ime_switch,
                "KEYCHAR" => &mut keychar,
                _ => continue,
            };
            if let Ok(expr) = Expr::parse(&sc.value) {
                for &ks in vk_to_keysyms(&sc.key) {
                    table.insert(ks, expr.clone());
                }
            }
        }
        // 한/영 키(0xff31)는 설정에 IME_SWITCH 가 없어도 기본 !A(0↔1) 로 동작.
        ime_switch
            .entry(KEY_HANGUL)
            .or_insert_with(|| Expr::parse("!A").expect("valid default switch expr"));

        let default_entry = config.default_entry.min(last);
        let mut engine = Self {
            entries,
            current: default_entry,
            default_entry,
            ime_switch,
            keychar,
            candidates: None,
            settings: Settings::default(), // apply_settings 가 곧 덮어쓴다
            pick_idx: 0,
            caps: 0,
            got_surrounding: false,
        };
        engine.apply_settings(st);
        engine
    }

    /// 사용자 설정을 반영한다(파생 필드와 현재 항목 보정). 항상 적용한다.
    fn apply_settings(&mut self, st: Settings) {
        let last = self.entries.len() - 1;
        let was_pick = self.settings.pick_entry;
        let pick_idx = st.entry.min(last);
        self.pick_idx = pick_idx;
        if st.pick_entry {
            // 항목 직접 지정: 고른 항목으로 고정(전환 없음).
            self.current = pick_idx;
        } else if was_pick {
            // 직접 지정 → 전체 전환: 기본 항목으로.
            self.current = self.default_entry.min(last);
        }
        self.settings = st;
    }

    /// config.ini 를 다시 읽어 바뀌었으면 반영한다. 바뀜 여부를 돌려준다.
    /// focus_in/enable 에서 호출 → 설정창 변경이 입력창 클릭 시 즉시 반영(재시작 불필요).
    fn reload_settings(&mut self) -> bool {
        let st = Settings::load();
        if st == self.settings {
            return false;
        }
        self.apply_settings(st);
        true
    }

    /// IME_SWITCH 키를 눌렀을 때 전환할 대상 항목 인덱스.
    /// 항목 직접 지정 모드에서는 전환하지 않고 현재 항목에 머문다(전환 단축글쇠 사용 불가).
    /// 그 외에는 ShortcutTable value 식(예 `!A`, A=현재 항목)을 평가. `!A`→0이면 1, 아니면 0.
    fn switch_target(&self, keyval: u32) -> usize {
        if self.settings.pick_entry {
            return self.current;
        }
        let len = self.entries.len() as i64;
        self.ime_switch
            .get(&keyval)
            .and_then(|e| {
                e.eval(&Ctx {
                    a: self.current as i64,
                    ..Default::default()
                })
                .ok()
            })
            .and_then(|v| match v {
                ExprValue::Int(t) => Some(t),
                _ => None,
            })
            .map(|t| t.clamp(0, len - 1) as usize)
            .unwrap_or(self.current)
    }

    fn cur(&self) -> &Mode {
        &self.entries[self.current]
    }

    /// 확정 문자열과 preedit 를 신호로 내보낸다.
    async fn emit(se: &SignalEmitter<'_>, commit: &str, preedit: &str) {
        if !commit.is_empty() {
            let _ = Self::commit_text(se, make_ibus_text(commit.to_string())).await;
        }
        let cursor = preedit.chars().count() as u32;
        let _ = Self::update_preedit_text(
            se,
            make_preedit_text(preedit.to_string()),
            cursor,
            !preedit.is_empty(),
            1, // IBusPreeditFocusMode::COMMIT: 포커스 잃을 때 클라이언트가 preedit 를
               // 그 자리에서 확정한다(엔진 전환 등으로 commit_text 가 누락되는 것 방지).
        )
        .await;
    }

    /// 포커스 상실/비활성화용: 조합 내부 상태만 비우고 **emit 하지 않는다**. 표시 중인
    /// preedit 는 클라이언트가 COMMIT 모드로 그 자리에서 확정하므로 우리가 지우면 안 된다
    /// (지우면 확정할 게 없어져 글자가 사라진다). 내부 core 만 비워 다음 포커스와 어긋나지
    /// 않게 한다.
    fn drop_composing(&mut self) {
        let i = self.current;
        if let Mode::Hangul(core) = &mut self.entries[i] {
            core.reset();
        }
    }

    /// 현재 항목이 한글 조합이면 조합을 확정해 내보낸다.
    async fn flush_current(&mut self, se: &SignalEmitter<'_>) {
        let i = self.current;
        if let Mode::Hangul(core) = &mut self.entries[i] {
            let commit = core.flush();
            if !commit.is_empty() {
                Self::emit(se, &commit, "").await;
            }
        }
    }

    /// 패널 심볼: 접두(한글=가, 로마자/직접=A)에 항목 번호를 아래첨자로 붙인다(예: "가₀").
    /// 항목 직접 지정 모드에서는 항목이 하나로 고정이라 번호 없이 접두만 보인다(예: "가", "A").
    fn mode_symbol(&self) -> String {
        let prefix = self.cur().symbol_prefix();
        if self.settings.pick_entry {
            prefix.to_string()
        } else {
            format!("{}{}", prefix, subscript_digits(self.current))
        }
    }

    /// 입력 모드 속성을 등록(패널이 심볼을 알도록). focus_in/enable 시 호출.
    /// 레이블("Geulbus 설정")은 패널 컨텍스트 메뉴에 뜨며, 누르면 property_activate 가 설정창을 연다.
    async fn register_props(&self, se: &SignalEmitter<'_>) {
        let _ = Self::register_properties(se, make_prop_list(&self.mode_symbol(), "Geulbus 설정"))
            .await;
    }

    /// 모드가 바뀌었을 때 패널 심볼을 갱신.
    async fn update_indicator(&self, se: &SignalEmitter<'_>) {
        let _ = Self::update_property(
            se,
            make_input_mode_property(&self.mode_symbol(), "Geulbus 설정"),
        )
        .await;
    }

    /// 키 이벤트를 분류한다(순수 함수). `process_key_event` 가 이 결과로 분기한다.
    /// IME_SWITCH 는 release/수식어보다 먼저 본다 — CapsLock 은 수식어 키심이기도 하므로.
    ///
    /// 인쇄 가능 키는 **물리 위치(keycode)** 를 US-QWERTY ASCII 로 환산해 KeyTable 을
    /// 조회한다(날개셋 모델). 그러면 사용자 XKB 가 드보락이어도 세벌식 자리가 고정된다.
    /// keycode 가 없거나(프로그램 주입) 매핑 밖이면 keyval 로 폴백한다.
    fn classify(&self, keyval: u32, keycode: u32, state: u32) -> KeyClass {
        // 단축키(IME_SWITCH)를 끄면 그 키들을 가로채지 않고 통과시킨다(아래 일반 분류로 진행).
        // 예: GNOME/Wayland 에서 CapsLock 을 직접 바인딩하려는 사용자.
        if self.settings.shortcuts_enabled && self.ime_switch.contains_key(&keyval) {
            return KeyClass::ImeSwitch;
        }
        // KEYCHAR(한자 키 등)도 release/수식어보다 먼저 본다 — 한자 대용 키(오른쪽 Ctrl 등)가
        // 수식어 키심이어도 동작하도록. 눌림/뗌 모두 이 분류로 가고, 처리부가 뗌을 소비한다.
        if self.keychar.contains_key(&keyval) {
            return KeyClass::Keychar;
        }
        if state & RELEASE_MASK != 0 {
            return KeyClass::Release;
        }
        if is_modifier_keysym(keyval) {
            return KeyClass::Modifier;
        }
        if state & SPECIAL_MODS != 0 {
            return KeyClass::ShortcutCombo;
        }
        if keyval == KEY_BACKSPACE {
            return KeyClass::Backspace;
        }
        let shift = state & SHIFT_MASK != 0;
        if let Some(ascii) = us_qwerty_ascii(keycode, shift) {
            return KeyClass::Printable(ascii);
        }
        if (0x20..=0x7e).contains(&keyval) {
            return KeyClass::Printable(keyval as u8); // keycode 없음 → keyval 폴백
        }
        KeyClass::FunctionKey
    }

    // ── 후보창(변환 모드) ────────────────────────────────────────────────────

    /// 후보창이 떠 있을 때 키 하나의 동작을 정한다(순수 상태 전이).
    /// 숫자 1~9 = 현재 페이지에서 선택, Enter/Space = 커서 후보 확정, Esc/Backspace = 취소,
    /// 방향키/PgUp/PgDn/한자 키 = 이동. 그 외 글쇠는 취소 후 일반 경로로 계속(Fallthrough).
    fn candidate_action(&mut self, class: KeyClass, keyval: u32, release: bool) -> CandAction {
        let st = self.candidates.as_mut().expect("candidate mode");
        if release {
            // 우리가 눌림을 소비한 키들의 뗌: 조용히 소비(응용에 홀로 된 뗌을 보내지 않는다).
            return CandAction::Consume;
        }
        match class {
            KeyClass::Modifier => CandAction::Pass,
            // 한자 키 재타: 다음 페이지(MS IME 동작).
            KeyClass::Keychar => {
                st.page_down();
                CandAction::Redraw
            }
            KeyClass::Printable(d @ b'1'..=b'9') => {
                let idx = st.page_start() + (d - b'1') as usize;
                match st.items.get(idx) {
                    Some(item) => CandAction::Commit(item.0.clone()),
                    None => CandAction::Consume, // 페이지 끝을 넘는 번호: 무시
                }
            }
            KeyClass::Printable(b' ') => CandAction::Commit(st.items[st.cursor].0.clone()),
            KeyClass::Backspace => CandAction::Cancel,
            KeyClass::FunctionKey => match keyval {
                KEY_RETURN | KEY_KP_ENTER => CandAction::Commit(st.items[st.cursor].0.clone()),
                KEY_ESCAPE => CandAction::Cancel,
                KEY_UP => {
                    st.up();
                    CandAction::Redraw
                }
                KEY_DOWN => {
                    st.down();
                    CandAction::Redraw
                }
                KEY_PAGE_UP | KEY_LEFT => {
                    st.page_up();
                    CandAction::Redraw
                }
                KEY_PAGE_DOWN | KEY_RIGHT => {
                    st.page_down();
                    CandAction::Redraw
                }
                _ => CandAction::Fallthrough,
            },
            // 다른 글쇠 입력·IME 전환·단축키: 변환을 취소하고 그 키를 정상 처리.
            _ => CandAction::Fallthrough,
        }
    }

    /// 후보창과 보조 텍스트(변환 대상 글자)를 (다시) 그린다.
    async fn show_candidates(&self, se: &SignalEmitter<'_>) {
        let Some(st) = &self.candidates else { return };
        let display: Vec<String> = st
            .items
            .iter()
            .map(|(text, gloss)| {
                if gloss.is_empty() {
                    text.clone()
                } else {
                    format!("{text} {gloss}") // 한자 후보는 훈음을 함께 표시
                }
            })
            .collect();
        let _ = Self::update_auxiliary_text(se, make_ibus_text(st.source.clone()), true).await;
        let _ = Self::update_lookup_table(se, make_lookup_table(&display, st.cursor as u32), true)
            .await;
    }

    /// 후보창과 보조 텍스트를 내린다.
    async fn hide_candidates(se: &SignalEmitter<'_>) {
        let _ = Self::hide_lookup_table(se).await;
        let _ = Self::hide_auxiliary_text(se).await;
    }

    /// 후보 `idx` 로 확정: 조합(원본 글자)을 버리고 후보 문자열을 확정 입력한다.
    async fn commit_candidate(&mut self, se: &SignalEmitter<'_>, idx: usize) {
        let Some(text) = self
            .candidates
            .as_ref()
            .and_then(|st| st.items.get(idx))
            .map(|item| item.0.clone())
        else {
            return;
        };
        if let Mode::Hangul(core) = &mut self.entries[self.current] {
            core.reset();
        }
        self.candidates = None;
        Self::hide_candidates(se).await;
        Self::emit(se, &text, "").await;
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngine {
    async fn process_key_event(
        &mut self,
        #[zbus(signal_emitter)] se: SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        // 진단: GEULBUS_DEBUG_KEYS=1 이면 받은 keyval/keycode/state 를 stderr 로 찍는다.
        // geulbus 활성 시 어떤 XKB 레이아웃 기준 keysym 이 오는지 확인용(드보락 vs us).
        if debug_keys_enabled() {
            let ch = char::from_u32(keyval)
                .filter(|c| !c.is_control())
                .unwrap_or(' ');
            eprintln!(
                "geulbus keyev: keyval=0x{keyval:04x} ({ch:?})  keycode=0x{keycode:x} ({keycode})  state=0x{state:x}"
            );
        }
        let class = self.classify(keyval, keycode, state);
        let release = state & RELEASE_MASK != 0;
        // 후보창(변환 모드)이 떠 있으면 키를 후보 탐색/선택으로 먼저 라우팅한다.
        if self.candidates.is_some() {
            match self.candidate_action(class, keyval, release) {
                CandAction::Pass => return Ok(false),
                CandAction::Consume => return Ok(true),
                CandAction::Redraw => {
                    self.show_candidates(&se).await;
                    return Ok(true);
                }
                CandAction::Commit(text) => {
                    if let Mode::Hangul(core) = &mut self.entries[self.current] {
                        core.reset(); // 조합(원본 글자)을 버리고 후보로 대체
                    }
                    self.candidates = None;
                    Self::hide_candidates(&se).await;
                    Self::emit(&se, &text, "").await;
                    return Ok(true);
                }
                CandAction::Cancel => {
                    self.candidates = None;
                    Self::hide_candidates(&se).await;
                    return Ok(true); // 조합(preedit)은 그대로 남아 이어서 편집
                }
                CandAction::Fallthrough => {
                    self.candidates = None;
                    Self::hide_candidates(&se).await;
                    // 취소 후 아래 일반 분기로 계속(예: 다른 자모 입력, IME 전환).
                }
            }
        }
        match class {
            // IME_SWITCH(한/영·CapsLock 등): 눌림/뗌 모두 소비, 눌림에서 전환식을 평가.
            // value 식(예 "!A")을 A=현재 항목으로 평가해 대상 항목을 얻는다. `!A` → 0이면 1, 아니면 0.
            KeyClass::ImeSwitch => {
                if !release {
                    let target = self.switch_target(keyval);
                    if target != self.current {
                        self.flush_current(&se).await;
                        self.current = target;
                        self.update_indicator(&se).await; // 패널 심볼(가N/AN) 갱신
                    }
                }
                Ok(true)
            }
            // KEYCHAR(한자 키 등): 식을 평가해 날개셋문자를 실행한다.
            // C0|0x82~0x85(후보 변환)면 조합 글자의 후보(특수문자/한자)창을 연다.
            KeyClass::Keychar => {
                if release {
                    return Ok(true); // 눌림을 소비했으니 뗌도 소비
                }
                let Some(val) = self
                    .keychar
                    .get(&keyval)
                    .and_then(|expr| expr.eval(&Ctx::default()).ok())
                else {
                    return Ok(false);
                };
                let fix_symbols = self.settings.fix_symbol_table;
                let i = self.current;
                // 로마자/직접 항목: 조합이 없어 변환 대상이 없으니 통과.
                let Mode::Hangul(core) = &mut self.entries[i] else {
                    return Ok(false);
                };
                match val {
                    ExprValue::Command(cmd) => {
                        let out = core.command(cmd);
                        if let Some(req) = out.convert {
                            // 후보 변환 요청: preedit 한 글자로 후보를 찾는다.
                            // 0x82(기본 한자 변환)만 내장 사전이 있다. 0x83~0x85(사용자
                            // 후보 2~4)는 데이터 계층 미구현 → 무동작(키만 소비).
                            let preedit = core.preedit();
                            let mut chars = preedit.chars();
                            if let (Some(ch), None, 0x82) = (chars.next(), chars.next(), req) {
                                let items = build_candidates(ch, fix_symbols);
                                if !items.is_empty() {
                                    self.candidates = Some(CandidateState {
                                        items,
                                        source: preedit,
                                        cursor: 0,
                                    });
                                    self.show_candidates(&se).await;
                                }
                            }
                            return Ok(true);
                        }
                        // 변환이 아닌 C0 명령(KEYCHAR 로 배당한 편집 특수글쇠 등).
                        Self::emit(&se, &out.commit, &out.preedit).await;
                        Ok(out.consumed)
                    }
                    // 날개셋문자가 일반 문자면: 조합 확정 후 그 문자를 입력.
                    ExprValue::Int(n) => {
                        let mut commit = core.flush();
                        if let Some(c) = u32::try_from(n).ok().and_then(char::from_u32) {
                            commit.push(c);
                        }
                        Self::emit(&se, &commit, "").await;
                        Ok(true)
                    }
                    ExprValue::Unit(_) => Ok(false),
                }
            }
            // 뗌·수식어 키 자체: 조합에 영향 없이 통과.
            KeyClass::Release | KeyClass::Modifier => Ok(false),
            // Ctrl/Alt/Super/Meta 조합(단축키): 조합만 확정하고 응용에 넘긴다.
            // 단축키 레이아웃은 IME 가 아니라 XKB(사용자 레이아웃)의 몫 — Wayland 에서
            // IME 의 ForwardKeyEvent 로 키 위치를 바꾸는 건 불가하므로 흉내 내지 않는다.
            // 사용자가 드보락 단축키를 원하면 자신의 키보드 레이아웃을 드보락으로 둔다.
            KeyClass::ShortcutCombo => {
                self.flush_current(&se).await;
                Ok(false)
            }
            // 나머지는 현재 항목의 방식에 따라 처리.
            KeyClass::Backspace | KeyClass::Printable(_) | KeyClass::FunctionKey => {
                let caps = state & LOCK_MASK != 0;
                // surrounding-text 가 실제로 동작하는 앱인가(capability 비트만으론 부족해
                // SetSurroundingText 수신까지 확인). 앞 글자 삭제/결합 특수글쇠의 전제.
                let supports_surround =
                    self.caps & CAP_SURROUNDING_TEXT != 0 && self.got_surrounding;
                let i = self.current;
                match &mut self.entries[i] {
                    // 한글 조합 항목.
                    Mode::Hangul(core) => {
                        // 앞 글자 결합 특수글쇠(C0 낱자 재결합·앞으로 이동)는 surrounding-text
                        // 지원 시에만 동작하도록 core 에 현재 지원 여부를 알린다.
                        core.set_surrounding_ok(supports_surround);
                        match class {
                            KeyClass::Backspace => {
                                // 조합 중이 아니고 BkspAttach 도 불가능하면 응용에 넘긴다.
                                if core.is_empty() && !supports_surround {
                                    return Ok(false);
                                }
                                let out = core.backspace();
                                // BkspAttach: 앞의 확정 글자를 응용에서 지운다(지원 시).
                                if out.delete_before > 0 {
                                    if supports_surround {
                                        let _ = Self::delete_surrounding_text(
                                            &se,
                                            -(out.delete_before as i32),
                                            out.delete_before,
                                        )
                                        .await;
                                    } else {
                                        // 못 지우면 되살리기가 무의미 → 통상 백스페이스로 폴백.
                                        core.reset();
                                        return Ok(false);
                                    }
                                }
                                if !out.consumed && out.delete_before == 0 {
                                    return Ok(false);
                                }
                                Self::emit(&se, &out.commit, &out.preedit).await;
                                Ok(true)
                            }
                            KeyClass::Printable(ascii) => {
                                let out = core.press(ascii, caps);
                                // C0 앞 글자 결합 특수글쇠: 앱의 옛 앞 글자를 지운다(지원 시).
                                if out.delete_before > 0 {
                                    if supports_surround {
                                        let _ = Self::delete_surrounding_text(
                                            &se,
                                            -(out.delete_before as i32),
                                            out.delete_before,
                                        )
                                        .await;
                                    } else {
                                        core.reset();
                                        return Ok(false);
                                    }
                                }
                                Self::emit(&se, &out.commit, &out.preedit).await;
                                Ok(out.consumed)
                            }
                            _ => {
                                // 기능키: 조합 확정 후 통과.
                                let commit = core.flush();
                                if !commit.is_empty() {
                                    Self::emit(&se, &commit, "").await;
                                }
                                Ok(false)
                            }
                        }
                    }
                    // 로마자/직접 항목: KeyTable 로 문자만 내보내고, 매핑 없으면 패스스루.
                    Mode::Latin { keys } => {
                        if let KeyClass::Printable(ascii) = class {
                            if let Some(expr) = keys.get(&(ascii as u32)) {
                                let ctx = Ctx {
                                    p: caps as i64,
                                    ..Default::default()
                                };
                                if let Ok(ExprValue::Int(n)) = expr.eval(&ctx) {
                                    if let Some(ch) = u32::try_from(n).ok().and_then(char::from_u32)
                                    {
                                        Self::emit(&se, &ch.to_string(), "").await;
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                        Ok(false) // 매핑 없음 / 백스페이스 / 기능키 → 응용에 넘김
                    }
                }
            }
        }
    }

    async fn focus_in(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) -> fdo::Result<()> {
        // 설정창 변경을 즉시 반영(재시작 불필요): 입력 컨텍스트가 잡힐 때마다 config.ini 재확인.
        self.reload_settings();
        self.register_props(&se).await;
        // 새 입력 컨텍스트: surrounding-text 수신 상태 초기화 후 요청. 실제로 SetSurroundingText
        // 가 와야 BkspAttach 를 시도한다(capability 비트는 거짓일 수 있음).
        self.got_surrounding = false;
        if self.caps & CAP_SURROUNDING_TEXT != 0 {
            let _ = Self::require_surrounding_text(&se).await;
        }
        Ok(())
    }

    async fn focus_out(
        &mut self,
        #[zbus(signal_emitter)] se: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        // 후보창이 뜬 채 포커스가 떠나면 변환을 취소한다(상태가 남으면 다음 컨텍스트의
        // 키가 후보 탐색으로 새어 들어간다).
        if self.candidates.take().is_some() {
            Self::hide_candidates(&se).await;
        }
        // preedit 는 그대로 두고(클라이언트가 COMMIT 모드로 확정) 내부만 비운다.
        self.drop_composing();
        Ok(())
    }

    async fn reset(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) -> fdo::Result<()> {
        // 명시적 Reset(취소): 조합을 비우고 preedit 도 지운다. (엔진 전환은 Reset 이 아니라
        // focus_out 경로로 오며, 거기선 COMMIT 모드로 글자를 보존한다.)
        if self.candidates.take().is_some() {
            Self::hide_candidates(&se).await;
        }
        self.drop_composing();
        Self::emit(&se, "", "").await;
        Ok(())
    }

    async fn enable(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) -> fdo::Result<()> {
        self.reload_settings();
        self.register_props(&se).await;
        self.got_surrounding = false;
        if self.caps & CAP_SURROUNDING_TEXT != 0 {
            let _ = Self::require_surrounding_text(&se).await;
        }
        Ok(())
    }

    /// 데몬/응용이 surrounding-text 를 전달(RequireSurroundingText 응답). 실제로 한 번이라도
    /// 오면 이 앱은 surrounding-text 를 진짜 지원하는 것으로 보고 BkspAttach 를 허용한다.
    fn set_surrounding_text(&mut self, _text: Value<'_>, _cursor_pos: u32, _anchor_pos: u32) {
        self.got_surrounding = true;
    }

    async fn disable(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) -> fdo::Result<()> {
        // focus_out 과 동일: preedit 는 클라이언트가 COMMIT, 내부만 비운다.
        if self.candidates.take().is_some() {
            Self::hide_candidates(&se).await;
        }
        self.drop_composing();
        Ok(())
    }

    fn set_capabilities(&mut self, caps: u32) {
        self.caps = caps;
    }

    fn set_cursor_location(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    fn property_activate(&mut self, name: String, _state: u32) {
        // 패널 컨텍스트 메뉴에서 InputMode 속성(설정 항목)을 누르면 설정창을 띄운다.
        if name == "InputMode" {
            let _ = std::process::Command::new("geulbus-setup")
                .spawn()
                .or_else(|_| std::process::Command::new("/usr/local/bin/geulbus-setup").spawn());
        }
    }

    // 패널(마우스 휠·버튼)에서 오는 후보창 탐색. 키보드와 같은 상태 머신을 쓴다.
    async fn page_up(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) {
        if let Some(st) = &mut self.candidates {
            st.page_up();
            self.show_candidates(&se).await;
        }
    }
    async fn page_down(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) {
        if let Some(st) = &mut self.candidates {
            st.page_down();
            self.show_candidates(&se).await;
        }
    }
    async fn cursor_up(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) {
        if let Some(st) = &mut self.candidates {
            st.up();
            self.show_candidates(&se).await;
        }
    }
    async fn cursor_down(&mut self, #[zbus(signal_emitter)] se: SignalEmitter<'_>) {
        if let Some(st) = &mut self.candidates {
            st.down();
            self.show_candidates(&se).await;
        }
    }
    /// 패널에서 후보를 클릭. `index` 는 현재 페이지 안 위치(0부터).
    async fn candidate_clicked(
        &mut self,
        #[zbus(signal_emitter)] se: SignalEmitter<'_>,
        index: u32,
        _button: u32,
        _state: u32,
    ) {
        let Some(st) = &self.candidates else { return };
        let idx = st.page_start() + index as usize;
        self.commit_candidate(&se, idx).await;
    }

    // ── 신호(engine → daemon) ────────────────────────────────────────────────

    #[zbus(signal)]
    async fn commit_text(se: &SignalEmitter<'_>, text: Value<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        se: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    /// 패널에 속성(입력 모드 표시기) 목록을 등록.
    #[zbus(signal)]
    async fn register_properties(se: &SignalEmitter<'_>, props: Value<'_>) -> zbus::Result<()>;

    /// 모드 변경 시 패널 속성(심볼)을 갱신.
    #[zbus(signal)]
    async fn update_property(se: &SignalEmitter<'_>, prop: Value<'_>) -> zbus::Result<()>;

    /// 커서 앞뒤 글자를 응용에서 지운다(BkspAttach 용). offset<0 이면 커서 앞.
    /// `(offset: i32, nchars: u32)`.
    #[zbus(signal)]
    async fn delete_surrounding_text(
        se: &SignalEmitter<'_>,
        offset: i32,
        nchars: u32,
    ) -> zbus::Result<()>;

    /// 응용에 surrounding-text 를 보내달라고 요청. 응답은 SetSurroundingText 메서드로 온다.
    #[zbus(signal)]
    async fn require_surrounding_text(se: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// 후보창 갱신. `table` 은 IBusLookupTable(v), `visible` 이 true 면 패널이 띄운다.
    #[zbus(signal)]
    async fn update_lookup_table(
        se: &SignalEmitter<'_>,
        table: Value<'_>,
        visible: bool,
    ) -> zbus::Result<()>;

    /// 후보창 닫기.
    #[zbus(signal)]
    async fn hide_lookup_table(se: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// 후보창 위 보조 텍스트(변환 대상 글자) 갱신. `text` 는 IBusText(v).
    #[zbus(signal)]
    async fn update_auxiliary_text(
        se: &SignalEmitter<'_>,
        text: Value<'_>,
        visible: bool,
    ) -> zbus::Result<()>;

    /// 보조 텍스트 닫기.
    #[zbus(signal)]
    async fn hide_auxiliary_text(se: &SignalEmitter<'_>) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use geulbus_core::Config;

    // VK_HANGUL 과 VK_CAPITAL 을 IME_SWITCH 로 둔 최소 설정.
    const MINI: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<EditContextSetting version="0x500">
  <EditorLayer flag="0">
    <ShortcutTable>
      <Shortcut key="VK_HANGUL" usage="IME_SWITCH" value="!A"/>
      <Shortcut key="VK_CAPITAL" modifier="DONT_EAT|KEEP_LAMP" usage="IME_SWITCH" value="!A"/>
      <Shortcut key="VK_HANJA" usage="KEYCHAR" value="C0|0x82"/>
    </ShortcutTable>
    <FinalConvTable><FinalConv from="0x1100" to="0x3131"/></FinalConvTable>
  </EditorLayer>
  <InputLayer default="0" current="0">
    <InputEntry>
      <InputSchemeSetting object="CBasicInputScheme">
        <KeyTable name="mini" flag="0" from="33" to="126">
          <Key at="0x6B" value="H3|G_"/>
          <Key at="0x40" value="T ? H3|_RG : 0x40"/>
        </KeyTable>
      </InputSchemeSetting>
      <GeneratorSetting object="CNgsImeEx">
        <UnitMixTable/><VirtualUnitTable/><AutomataTable default="0"/>
      </GeneratorSetting>
    </InputEntry>
  </InputLayer>
</EditContextSetting>"#;

    /// 전역 config.ini 에 의존하지 않도록 기본 설정(전체 모드)으로 만든다.
    fn engine() -> IBusEngine {
        let cfg = Config::parse(MINI).unwrap();
        IBusEngine::with_settings(&cfg, Settings::default())
    }

    #[test]
    fn subscript_digits_maps_to_unicode() {
        assert_eq!(subscript_digits(0), "₀");
        assert_eq!(subscript_digits(2), "₂");
        assert_eq!(subscript_digits(10), "₁₀");
    }

    #[test]
    fn mode_symbol_full_uses_subscript() {
        // 전체 모드(기본): 접두 + 아래첨자 항목번호. MINI 는 한글 항목 1개, current=0.
        let e = engine();
        assert_eq!(e.mode_symbol(), "가₀");
    }

    #[test]
    fn mode_symbol_pick_has_no_number() {
        // 항목 직접 지정 모드: 번호 없이 접두만.
        let cfg = Config::parse(MINI).unwrap();
        let e = IBusEngine::with_settings(
            &cfg,
            Settings {
                pick_entry: true,
                entry: 0,
                shortcuts_enabled: true,
                fix_symbol_table: true,
            },
        );
        assert_eq!(e.mode_symbol(), "가");
    }

    #[test]
    fn classify_uses_keycode_not_keyval() {
        // 핵심: 인쇄 키는 keycode(물리 위치)로 분류한다. keyval 이 드보락이어도(예: 'p'=0x70)
        // keycode 19(물리 R)면 US-QWERTY 'r' 로 본다 → 세벌식 자리 고정.
        let e = engine();
        // keyval=0x70('p', 드보락), keycode=19(물리 R), modifier 없음 → Printable('r')
        assert_eq!(e.classify(0x70, 19, 0), KeyClass::Printable(b'r'));
        // Shift+물리2(keycode 3) → '@' (세벌식 shifted 자모 인덱스)
        assert_eq!(e.classify(0x32, 3, SHIFT_MASK), KeyClass::Printable(b'@'));
        // keycode 0(프로그램 주입) → keyval 폴백
        assert_eq!(e.classify(b'k' as u32, 0, 0), KeyClass::Printable(b'k'));
    }

    #[test]
    fn default_settings_is_full_mode() {
        let e = engine();
        assert!(!e.settings.pick_entry);
    }

    #[test]
    fn apply_pick_entry_sets_current() {
        let cfg = Config::parse(MINI).unwrap();
        let st = Settings {
            pick_entry: true,
            entry: 0,
            shortcuts_enabled: true,
            fix_symbol_table: true,
        };
        let e = IBusEngine::with_settings(&cfg, st);
        assert!(e.settings.pick_entry);
        assert_eq!(e.current, 0); // 직접 지정 → 고른 항목에서 고정
    }

    #[test]
    fn pick_entry_disables_switch() {
        // 항목 직접 지정 모드: 한/영 키를 눌러도 전환 대상이 현재 항목(=전환 없음).
        let cfg = Config::parse(MINI).unwrap();
        let e = IBusEngine::with_settings(
            &cfg,
            Settings {
                pick_entry: true,
                entry: 0,
                shortcuts_enabled: true,
                fix_symbol_table: true,
            },
        );
        assert_eq!(e.switch_target(0xff31), e.current);
    }

    #[test]
    fn reload_detects_change() {
        // 같은 설정이면 reload 가 false, 바뀌면 true 를 돌려준다(파일 IO 없이 내부 상태로).
        let mut e = engine();
        let before = e.settings;
        // settings 가 같으면(파일이 default 와 동일하거나 없으면) 변화 감지 안 함.
        // 여기선 apply_settings 로 직접 바꿔 동작만 확인.
        e.apply_settings(Settings {
            pick_entry: true,
            entry: 0,
            shortcuts_enabled: true,
            fix_symbol_table: true,
        });
        assert_ne!(before.pick_entry, e.settings.pick_entry);
        assert!(e.settings.pick_entry);
    }

    #[test]
    fn capslock_in_switch_set() {
        let e = engine();
        // 설정의 VK_CAPITAL → Caps_Lock(0xffe5), VK_HANGUL → 0xff31
        assert!(e.ime_switch.contains_key(&0xffe5));
        assert!(e.ime_switch.contains_key(&0xff31));
        // VK_HANJA 는 KEYCHAR 라 전환 집합에 없어야 한다.
        assert!(!e.ime_switch.contains_key(&0xff34));
    }

    #[test]
    fn switch_expr_is_not_a_toggle() {
        use geulbus_core::expr::{Ctx, Value as EV};
        let e = engine();
        // ShortcutTable value="!A" → A=현재 항목. 0이면 1, 아니면 0.
        let expr = e.ime_switch.get(&0xffe5).expect("capslock switch expr");
        let f = |cur: i64| match expr
            .eval(&Ctx {
                a: cur,
                ..Default::default()
            })
            .unwrap()
        {
            EV::Int(t) => t,
            other => panic!("expected int, got {other:?}"),
        };
        assert_eq!(f(0), 1);
        assert_eq!(f(1), 0);
        assert_eq!(f(2), 0);
    }

    #[test]
    fn shift_is_modifier_not_function_key() {
        let e = engine();
        // 버그 재현 방지: Shift 는 Modifier(통과)여야지, FunctionKey(조합 확정)면 안 된다.
        assert_eq!(e.classify(0xffe1, 0, 0), KeyClass::Modifier); // Shift_L
        assert_eq!(e.classify(0xffe2, 0, 0), KeyClass::Modifier); // Shift_R
    }

    #[test]
    fn capslock_classifies_as_ime_switch_even_on_release() {
        let e = engine();
        assert_eq!(e.classify(0xffe5, 0, 0), KeyClass::ImeSwitch);
        assert_eq!(e.classify(0xffe5, 0, RELEASE_MASK), KeyClass::ImeSwitch);
    }

    #[test]
    fn hangul_key_is_ime_switch() {
        assert_eq!(engine().classify(0xff31, 0, 0), KeyClass::ImeSwitch);
    }

    #[test]
    fn shortcuts_disabled_passes_switch_keys_through() {
        // shortcuts_enabled=false 면 한/영 키·CapsLock 을 가로채지 않고 통과(엔진 밖에서 직접 바인딩).
        let cfg = Config::parse(MINI).unwrap();
        let st = Settings {
            shortcuts_enabled: false,
            ..Settings::default()
        };
        let e = IBusEngine::with_settings(&cfg, st);
        // 한/영 키(0xff31): ImeSwitch 가 아니어야 함(기능키로 떨어져 통과).
        assert_ne!(e.classify(0xff31, 0, 0), KeyClass::ImeSwitch);
        // CapsLock(0xffe5): ImeSwitch 가 아니라 Modifier(통과)로 분류.
        assert_eq!(e.classify(0xffe5, 0, 0), KeyClass::Modifier);
        // 켜져 있으면 둘 다 ImeSwitch (대조군).
        assert_eq!(engine().classify(0xff31, 0, 0), KeyClass::ImeSwitch);
        assert_eq!(engine().classify(0xffe5, 0, 0), KeyClass::ImeSwitch);
    }

    #[test]
    fn at_key_with_shift_is_printable() {
        // 실키 Shift+물리2(keycode 3) → US-QWERTY '@' 로 분류 → 세벌식 ㄺ 조합 가능.
        assert_eq!(
            engine().classify(0x40, 3, SHIFT_MASK),
            KeyClass::Printable(b'@')
        );
    }

    #[test]
    fn ctrl_combo_is_shortcut() {
        // Ctrl+물리C(keycode 46) → 단축키(통과). keycode 있어도 ShortcutCombo 가 우선.
        assert_eq!(
            engine().classify(b'c' as u32, 46, CONTROL_MASK),
            KeyClass::ShortcutCombo
        );
    }

    #[test]
    fn super_combo_is_shortcut() {
        // Super+1 은 Mod4(1<<6)로 실려 온다. 단축키로 분류돼 통과해야 한다
        // (안 그러면 geulbus 이 '1' 을 입력으로 먹어 GNOME 단축키가 안 먹는다).
        assert_eq!(
            engine().classify(b'1' as u32, 2, MOD4_MASK),
            KeyClass::ShortcutCombo
        );
        // 가상 SUPER_MASK 로 실려 오는 환경도 동일.
        assert_eq!(
            engine().classify(b'1' as u32, 2, SUPER_MASK),
            KeyClass::ShortcutCombo
        );
    }

    #[test]
    fn release_of_normal_key_ignored() {
        assert_eq!(
            engine().classify(b'k' as u32, 37, RELEASE_MASK),
            KeyClass::Release
        );
    }

    #[test]
    fn backspace_classified() {
        assert_eq!(engine().classify(KEY_BACKSPACE, 14, 0), KeyClass::Backspace);
    }

    // ── 한자 키 · 후보창(변환 모드) ─────────────────────────────────────────

    #[test]
    fn hanja_key_is_keychar() {
        let e = engine();
        // MINI 의 <Shortcut key="VK_HANJA" usage="KEYCHAR" value="C0|0x82"/> 가 컴파일된다.
        assert_eq!(e.classify(0xff34, 0, 0), KeyClass::Keychar);
        // 뗌도 같은 분류로 와서 처리부가 소비한다(응용에 홀로 된 뗌을 보내지 않도록).
        assert_eq!(e.classify(0xff34, 0, RELEASE_MASK), KeyClass::Keychar);
    }

    #[test]
    fn keychar_expr_evaluates_to_convert_command() {
        use geulbus_core::expr::{Ctx, Value as EV};
        let e = engine();
        let expr = e.keychar.get(&0xff34).expect("hanja keychar expr");
        assert_eq!(expr.eval(&Ctx::default()).unwrap(), EV::Command(0x82));
    }

    #[test]
    fn build_candidates_symbols_and_hanja() {
        // 자음 → 특수문자 표. ㅁ 에 ※ 가 있고, 보정판(fix=true)은 끝에 ㉾ 가 추가된다.
        let fixed = build_candidates('ㅁ', true);
        assert!(fixed.iter().any(|(t, _)| t == "※"));
        assert_eq!(fixed.last().unwrap().0, "㉾");
        let orig = build_candidates('ㅁ', false);
        assert_eq!(orig.len() + 1, fixed.len());
        // ㄹ 4번째: 원본은 전각Ｆ(옛 MS IME 결함), 보정판은 °(날개셋 동작).
        assert_eq!(build_candidates('ㄹ', false)[3].0, "Ｆ");
        assert_eq!(build_candidates('ㄹ', true)[3].0, "°");
        // 음절 → 한자 사전(훈음 포함, 상용 한자 우선).
        let hak = build_candidates('학', true);
        assert_eq!(hak[0], ("學".to_string(), "배울 학".to_string()));
        // ㅉ(MS IME 도 미배당)과 비한글은 후보 없음 → 후보창을 열지 않는다.
        assert!(build_candidates('ㅉ', true).is_empty());
        assert!(build_candidates('A', true).is_empty());
    }

    #[test]
    fn candidate_state_navigation_wraps() {
        let mut st = CandidateState {
            items: (0..20).map(|i| (i.to_string(), String::new())).collect(),
            source: "ㅁ".into(),
            cursor: 0,
        };
        st.up();
        assert_eq!(st.cursor, 19); // 처음에서 위로: 끝으로 순환
        st.down();
        assert_eq!(st.cursor, 0);
        st.page_down();
        assert_eq!(st.cursor, 9);
        st.page_down();
        assert_eq!(st.cursor, 18);
        st.page_down();
        assert_eq!(st.cursor, 0); // 마지막 페이지에서 아래로: 첫 페이지로 순환
        st.page_up();
        assert_eq!(st.cursor, 18); // 첫 페이지에서 위로: 마지막 페이지 첫 후보
        assert_eq!(st.page_start(), 18);
    }

    #[test]
    fn candidate_action_select_cancel_fallthrough() {
        let mut e = engine();
        e.candidates = Some(CandidateState {
            items: vec![
                ("※".to_string(), String::new()),
                ("★".to_string(), String::new()),
            ],
            source: "ㅁ".to_string(),
            cursor: 0,
        });
        // 숫자 2 → 현재 페이지 2번째 후보 확정.
        assert_eq!(
            e.candidate_action(KeyClass::Printable(b'2'), 0, false),
            CandAction::Commit("★".to_string())
        );
        // Enter → 커서 후보 확정.
        assert_eq!(
            e.candidate_action(KeyClass::FunctionKey, KEY_RETURN, false),
            CandAction::Commit("※".to_string())
        );
        // Esc → 취소(조합 유지).
        assert_eq!(
            e.candidate_action(KeyClass::FunctionKey, KEY_ESCAPE, false),
            CandAction::Cancel
        );
        // 페이지 밖 번호 → 소비만(무동작).
        assert_eq!(
            e.candidate_action(KeyClass::Printable(b'9'), 0, false),
            CandAction::Consume
        );
        // 다른 글쇠 입력 → 취소 후 일반 경로로 계속.
        assert_eq!(
            e.candidate_action(KeyClass::Printable(b'k'), 0, false),
            CandAction::Fallthrough
        );
        // 뗌은 조용히 소비.
        assert_eq!(
            e.candidate_action(KeyClass::Release, 0, true),
            CandAction::Consume
        );
        // 한자 키 재타 → 페이지 이동(다시 그림).
        assert_eq!(
            e.candidate_action(KeyClass::Keychar, 0xff34, false),
            CandAction::Redraw
        );
        // 수식어는 통과(후보창 유지).
        assert_eq!(
            e.candidate_action(KeyClass::Modifier, 0xffe1, false),
            CandAction::Pass
        );
    }
}
