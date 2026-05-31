//! 사용자 설정(`~/.config/presguel/config.ini`) 읽기.
//!
//! 형식은 `key=value`(한 줄에 하나, `#` 주석 무시) — addr.rs 의 IBus 주소 파일과 같은
//! 단순 형식이라 의존성이 없고, Python 설정창도 쉽게 읽고 쓴다.
//!
//! ```ini
//! # 입력 항목 직접 지정: 켜면 아래에서 고른 InputEntry 하나만 쓴다(항목 전환 단축키 없음).
//! # 끄면(기본) 날개셋 설정의 모든 InputEntry 를 쓰고, 항목 전환은 ShortcutTable 에
//! # 등록된 IME_SWITCH 단축글쇠가 있을 때만 동작한다.
//! pick_entry = false
//! # pick_entry 가 켜졌을 때 쓸 InputEntry 인덱스.
//! entry = 0
//! # IME_SWITCH 단축글쇠(한/영 키 등)로 입력 항목을 전환할지. 끄면 그 키를 통과시킨다.
//! shortcuts_enabled = true
//! ```

use std::path::PathBuf;

/// 파싱된 사용자 설정. 파일이 없거나 키가 빠지면 기본값(날개셋과 동일 동작).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// false(기본): 모든 InputEntry 를 읽어 날개셋과 똑같이 동작(항목 전환은 ShortcutTable
    /// 의 IME_SWITCH 단축글쇠가 등록됐을 때만).
    /// true: 아래 `entry` 하나만 써서 고정(항목 전환 단축글쇠 사용 불가).
    pub pick_entry: bool,
    /// `pick_entry` 가 켜졌을 때 쓸 InputEntry 인덱스(항목 수로 클램프).
    pub entry: usize,
    /// IME_SWITCH 단축글쇠(한/영 키 등)로 입력 항목을 전환할지. 기본 켜짐.
    /// 끄면 그 키들을 가로채지 않고 통과시켜, 사용자가 엔진 밖(GNOME/XKB 등)에서
    /// 직접 바인딩할 수 있다. 참고: Wayland 에서 CapsLock 은 컴포지터가 직접 처리해
    /// IME 까지 오지 않으므로, 애초에 단축글쇠로 쓸 수 없다(직접 바인딩해야 함).
    /// (`pick_entry` 가 켜지면 전환 자체가 없으므로 이 값은 무의미해진다.)
    pub shortcuts_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pick_entry: false,
            entry: 0,
            shortcuts_enabled: true,
        }
    }
}

impl Settings {
    /// 표준 경로(`$PRESGUEL_CONFIG_INI` 또는 `~/.config/presguel/config.ini`)에서 읽는다.
    /// 파일이 없으면 기본값.
    pub fn load() -> Self {
        match Self::path().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(body) => Self::parse(&body),
            None => Self::default(),
        }
    }

    /// 설정 파일 경로.
    pub fn path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("PRESGUEL_CONFIG_INI") {
            if !p.is_empty() {
                return Some(PathBuf::from(p));
            }
        }
        let base = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })?;
        Some(base.join("presguel").join("config.ini"))
    }

    /// `key=value` 본문을 파싱. 알 수 없는 키/값은 무시하고 기본값 유지.
    pub fn parse(body: &str) -> Self {
        let mut s = Self::default();
        for line in body.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "pick_entry" => {
                    if let Some(b) = parse_bool(v) {
                        s.pick_entry = b;
                    }
                }
                "entry" => {
                    if let Ok(n) = v.parse() {
                        s.entry = n;
                    }
                }
                "shortcuts_enabled" => {
                    if let Some(b) = parse_bool(v) {
                        s.shortcuts_enabled = b;
                    }
                }
                _ => {}
            }
        }
        s
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_full_mode() {
        let s = Settings::default();
        assert!(!s.pick_entry);
    }

    #[test]
    fn parse_basic() {
        let s = Settings::parse("pick_entry = true\nentry = 2\n");
        assert!(s.pick_entry);
        assert_eq!(s.entry, 2);
    }

    #[test]
    fn parse_ignores_comments_and_unknown() {
        let s = Settings::parse("# 주석\npick_entry=on\nfoo=bar\n\n");
        assert!(s.pick_entry);
        assert_eq!(s.entry, 0); // 기본값 유지
    }

    #[test]
    fn parse_empty_is_default() {
        assert_eq!(Settings::parse(""), Settings::default());
    }

    #[test]
    fn bool_forms() {
        assert!(Settings::parse("pick_entry=1").pick_entry);
        assert!(Settings::parse("pick_entry=yes").pick_entry);
        assert!(!Settings::parse("pick_entry=off").pick_entry);
        assert!(!Settings::parse("pick_entry=garbage").pick_entry);
    }

    #[test]
    fn shortcuts_enabled_default_on_and_parses() {
        // 기본은 단축키 사용.
        assert!(Settings::default().shortcuts_enabled);
        assert!(Settings::parse("").shortcuts_enabled);
        // 끄기/켜기.
        assert!(!Settings::parse("shortcuts_enabled = false").shortcuts_enabled);
        assert!(!Settings::parse("shortcuts_enabled=off").shortcuts_enabled);
        assert!(Settings::parse("shortcuts_enabled = on").shortcuts_enabled);
    }
}
