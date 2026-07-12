# geulbus

**날개셋(nalgaeset) 입력 설정과 호환되는, 순수 Rust로 작성하는 ibus 한글 입력기.**

> 🟡 **초기 동작본.** 세벌식-맞춤 한글 조합이 ibus에서 실제로 동작합니다(완성형 +
> 옛한글 첫가끝 + 겹낱자/갈마들이 + 한자·특수문자 변환). 다듬을 부분이 남아
> 있습니다(로마자 드보락 항목, 일부 제어 명령 등).

---

## 무엇인가

[날개셋 한글 입력기](http://moogi.new21.org/)의 "입력 설정" XML(`nalgaeset.xml`)을
해석하여, Linux의 [ibus](https://github.com/ibus/ibus) 환경에서 **동일한 한글 입력
동작을 재현**하는 입력기 엔진입니다.

- **범용 해석기**: 특정 자판을 하드코딩하지 않고, 임의의 날개셋 설정 XML
  (`KeyTable` / `UnitMixTable` / `VirtualUnitTable` / `AutomataTable` /
  `FinalConvTable` 등)을 파싱·해석하는 것을 목표로 합니다.
- **순수 Rust**: `libibus`(C) 의존 없이 [`zbus`](https://github.com/dbus2/zbus)로
  ibus 데몬과 D-Bus를 직접 주고받습니다.
- **옛한글 완전 지원**: 첫가끝(U+1100 조합용 자모) 조합과 `FinalConvTable`을
  포함합니다. 옛한글 음절의 *표시*는 옛한글 OpenType 폰트(예: 함초롬)가 필요하며,
  이는 사용자 폰트 환경의 몫입니다. 엔진은 올바른 코드포인트를 commit 합니다.

## 배경

날개셋은 강력하지만 Windows 중심입니다. 같은 세벌식(및 임의 사용자 설정) 경험을
Linux ibus 위에서 그대로 쓰기 위해 만듭니다. 참고로 호환 대상 설정 파일은 이
저장소 바깥(사용자 환경)에 있는 사용자 소유 파일이며, 저장소에 포함하지 않습니다.

## 호환 대상 포맷

`EditContextSetting` (날개셋 입력 설정):

| 요소 | 역할 |
| --- | --- |
| `ShortcutTable` | 한/영 전환 등 특수키 동작 |
| `FinalConvTable` | 미완성/홑낱자 출력 시 조합용·옛 자모 → 호환 자모 변환 |
| `KeyTable` | 물리 키 → 자모 단위/문자 (값-식 언어) |
| `UnitMixTable` | 낱자 조합 규칙 (겹받침, 겹모음, 된소리, 갈마들이) |
| `VirtualUnitTable` | 가상 단위 |
| `AutomataTable` | 한글 조합 오토마타 (상태 전이 식) |
| `Extra` / `Bksp` | 백스페이스 동작 |

## 구조 (예정)

- **`geulbus-core`** — 설정 파서 + 값-식 평가기 + 자모 모델 + 한글 오토마타.
  ibus와 무관한 순수 라이브러리. 테스트 벡터로 동작을 검증합니다.
- **`geulbus`** — `zbus` 기반 ibus 프런트엔드(Factory / Engine),
  preedit·commit, 한/영 전환.

## 진행 상황

- [x] 날개셋 포맷·식 언어·오토마타 해석 명세 (`research/01-nalgaeset-format.md`)
- [x] 설정 파일 역공학 → 동작 명세·테스트 벡터(오라클) (`research/02-config-decode.md`)
- [x] `geulbus-core`: 파서 / 식 평가 / 자모 / 오토마타 (단위·통합 테스트 통과)
- [x] `geulbus`: zbus 프런트엔드 (실제 ibus 데몬에서 조합 검증)
- [x] 옛한글(첫가끝) 조합·출력
- [x] 설치 / 패키징(ibus 컴포넌트 등록, `scripts/install.sh`)
- [x] 한/영 전환(한글 키 / CapsLock, ShortcutTable 해석) + 패널 표시기(날개셋 방식 `가N`/`AN`)
- [x] 입력 항목 전환(IME_SWITCH 의 `!A` 식 평가: 0이면 1, 아니면 0)
- [x] 설정창(GTK4/libadwaita, 즉시 적용) + 입력 항목 직접 지정
- [x] 레이아웃 독립 한글 입력: keycode(물리 위치) 기준이라 QWERTY/드보락 무관하게 자판 고정
- [x] 키보드 배열별 엔진 등록(Dvorak/Colemak/Workman 등): 단축키·영문이 그 배열을 따름
- [x] 한자·특수문자 변환(한자 키, `C0|0x82`): 자음 → MS IME식 특수문자 표, 음절 → 한자
      후보(훈음 표시). 후보창은 날개셋 규약(숫자 선택, Space/화살표 이동, A~Z 페이지 점프,
      Tab 확장 모드, Ctrl/Shift+엔터 漢(한)/한(漢) 형식 삽입, Esc/한자 취소)
- [ ] 사용자 정의 후보 변환 2~4(`C0|0x83`~`0x85`): 보류.
      [#1](https://github.com/chaotic-ground/geulbus/issues/1) 참고
- [ ] 3개 이상 항목을 직접 고르는 전환 글쇠
- [ ] 백스페이스 동작 방식 세분화(Bksp 표 충실 반영)

## 빌드 / 설치

```sh
# 빌드 + 테스트
cargo test

# 설치 (이 환경의 ibus 는 시스템 컴포넌트 디렉터리만 스캔하므로 sudo 필요)
#   - 바이너리 → /usr/local/bin/geulbus
#   - 자판     → ~/.config/geulbus/layout.xml  (인자로 경로 지정 가능)
#               + 시스템 기본값 /usr/local/share/geulbus/layout.xml (fallback)
#   - 컴포넌트 → /usr/share/ibus/component/geulbus.xml
scripts/install.sh /path/to/layout.xml

# 입력기 전환
ibus engine geulbus        # 또는 GNOME 설정 → 키보드 → 입력 소스에서 추가 후 Super+Space
```

### 한/영 전환과 표시기

- `IME_SWITCH` 글쇠(**한글 키 / CapsLock** 등)는 그 단축글쇠의 `value` 식(예 `!A`)을 평가해
  대상 입력 항목을 정합니다. 식의 변수 `A` = 현재 항목 인덱스이며, `!A` 는 "0이면 1, 0이
  아니면 0"(0↔1 토글)입니다.
- **한글 입력은 키보드 레이아웃과 무관합니다.** 자판을 물리 키 위치(keycode)로 잡으므로,
  시스템 레이아웃이 QWERTY든 드보락이든 세벌식 자리가 고정됩니다(날개셋이 스캔코드로 하던
  방식과 동일).
- **단축키(Ctrl/Alt/Super+키)는 그대로 통과시킵니다.** 단축키·영문 배열은 입력기가 아니라
  XKB 레이아웃이 정합니다. GNOME/Wayland 에서는 ibus 엔진이 활성일 때 쓰는 XKB 레이아웃을
  **컴포넌트 XML 의 `<layout>`/`<layout_variant>`** 가 결정합니다(근거:
  `research/07-ibus-engine-xkb-pairing.md`).
- 그래서 geulbus 은 **키보드 배열별 엔진을 여러 개 등록**합니다:
  `Geulbus`(QWERTY), `Geulbus (Dvorak)`, `Geulbus (Colemak)`, `Geulbus (Workman)` 등
  영문 대체배열 전부. GNOME 설정 → 키보드 → 입력 소스 → 한국어에서 **원하는 배열의 항목을
  고르면** 그 배열이 단축키·영문에 그대로 적용됩니다. (한글 자판 자체는 keycode 기준이라
  어느 항목을 골라도 동일합니다.) 별도 권한·재설정이 필요 없습니다.
- 패널 표시기는 날개셋(Windows) 방식을 따라 **`접두 + 항목번호`** 로 동적 표시됩니다
  (한글 항목 `가N`, 로마자/직접 항목 `AN`; 예 `가0`, `A1`). 항목을 XML 에 추가하면
  번호도 늘어납니다. (ibus 속성 `RegisterProperties`/`UpdateProperty`.)
- **CapsLock 을 한/영 키로** (GNOME/Wayland): 컴포지터가 CapsLock 잠금을 가로채므로
  XKB 레벨 재매핑이 필요합니다. 아래 스크립트가 *CapsLock 단독 → 한/영*,
  *Shift+CapsLock → 평소 대소문자 잠금* 으로 설정합니다(sudo 불필요):

  ```sh
  scripts/setup-capslock-hangul.sh          # 적용 (입력 소스 한 번 전환하면 반영)
  scripts/setup-capslock-hangul.sh --revert # 되돌리기
  ```

자판 파일은 다음 순서로 찾습니다(존재하는 첫 파일): `GEULBUS_CONFIG` 환경변수 →
`~/.config/geulbus/layout.xml` → `~/.config/geulbus/nalgaeset.xml`(옛 이름, 하위 호환)
→ `<XDG_DATA_DIRS>/geulbus/layout.xml`(시스템 기본값).

### 설정창 / 입력 항목 직접 지정

GNOME 입력 소스 목록에서 geulbus 옆 **톱니바퀴(설정)** 를 누르면 설정창이 뜹니다
(또는 `geulbus-setup` 실행). 설정은 `~/.config/geulbus/config.ini` 에 저장됩니다.

- **직접 지정 끔(기본)**: 설정의 **모든 InputEntry** 를 읽어 날개셋과 똑같이 동작합니다.
  항목 전환은 `ShortcutTable` 에 등록된 전환 단축글쇠(`IME_SWITCH`)가 있을 때만 동작합니다.
- **직접 지정 켬**: 드롭다운으로 **사용할 입력 항목** 하나를 골라 고정합니다. 이 경우 항목
  전환 단축글쇠는 사용할 수 없습니다.
- **단축글쇠 사용** 토글: 끄면 한/영 키 등 전환 글쇠를 엔진이 가로채지 않고 통과시켜,
  사용자가 직접 바인딩할 수 있습니다. (참고: Wayland 에서 CapsLock 은 컴포지터가 직접
  처리해 입력기까지 오지 않으므로 단축글쇠로 쓸 수 없고, GNOME 설정 등에서 직접 지정해야
  합니다.)
- 영문 배열·단축키 배열은 GNOME 입력 소스에서 `Geulbus (Dvorak)` 처럼 배열별 항목을 골라
  정합니다(한글 자판은 keycode 기준이라 배열 무관하게 고정).

### 한자·특수문자 입력 (한자 키)

조합 중 **한자 키**(ShortcutTable 의 `usage="KEYCHAR"`, 기본 `VK_HANJA` → `C0|0x82`)를
누르면 후보창이 뜹니다. Windows 에서 하던 그대로입니다:

- **자음 + 한자** (예: `ㅁ` + 한자): MS IME 식 특수문자 표(ㄱ=문장부호, ㄷ=수학,
  ㅁ=도형기호, ㅎ=그리스 문자 등 18키, ㅉ 없음). 데이터는
  [okpyeon](https://github.com/chaotic-ground/okpyeon) 크레이트(libhangul 유래).
- **음절 + 한자** (예: `학` + 한자): 한자 후보(훈음 표시, 상용 한자 우선).
- 후보창 키는 날개셋 규약: 숫자 1~9 선택, Enter 확정, Space/화살표 한 칸 이동,
  PgUp/PgDn 페이지, Home/End 첫/끝, A~Z 페이지 점프, Tab 확장 모드,
  Ctrl+엔터/번호 `漢(한)` 형식, Shift+엔터/번호 `한(漢)` 형식, Esc/한자 취소.
- 설정창의 **특수문자 표 결함 보정**(기본 켬): 켜면 ㄹ 4번째가 `°`(날개셋 방식)이고
  ㅁ 끝에 `㉾`(최신 Windows 방식), 끄면 옛 MS IME 원본 표 그대로
  (`config.ini` 의 `fix_symbol_table`).
- **전용 한자 키가 없는 키보드**(대부분의 노트북, "한/영" 각인 키가 물리적으로는
  오른쪽 Alt): `scripts/setup-hanja-key.sh` 로 오른쪽 Alt 를 한자 키로 만들 수
  있습니다(sudo 불필요, `--revert` 로 되돌리기). 표준 XKB 옵션 `korean:ralt_hanja`
  는 그룹 1에만 적용돼 dvorak-mac 등과 짝지어진 소스에서 먹지 않으므로, 이
  스크립트는 사용자 XKB rules 의 `:all` 지정자로 모든 그룹에 적용합니다.
- 사용자 정의 후보 변환 2~4(`C0|0x83`~`0x85`)는 아직 지원하지 않습니다
  ([#1](https://github.com/chaotic-ground/geulbus/issues/1)). 단어 단위 변환은
  계획에 없습니다(날개셋도 Windows MS IME 사전에 위임하는 기능).

### 동작 확인 (GUI 없이)

실행 중인 엔진을 D-Bus 로 직접 구동해 조합 결과를 확인하는 예제:

```sh
cargo run -p geulbus --example drive -- "kf kfhf"   # → "가가나"
```

## 라이선스

다음 두 라이선스 중 원하는 쪽을 선택해 사용할 수 있습니다.

- MIT ([LICENSE-MIT](./LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))

명시적으로 달리 밝히지 않는 한, 이 저장소에 제출된 기여물은 위 두 라이선스로
제공되는 것에 동의한 것으로 간주합니다.
