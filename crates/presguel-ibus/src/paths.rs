//! 설정·자판 파일 경로 해석. XDG 규약을 따르며 여러 후보를 순서대로 찾는다.
//! main.rs(자판)·settings.rs(config.ini)·addr.rs(ibus 주소 base)가 공유해, XDG
//! base 계산이 여러 곳에 중복되지 않도록 한다.

use std::path::PathBuf;

/// presguel 설정/데이터 디렉터리 이름.
const APP_DIR: &str = "presguel";
/// 자판(입력 설정) 파일의 표준 이름과 옛 이름(하위 호환).
const LAYOUT_FILE: &str = "layout.xml";
const LAYOUT_FILE_LEGACY: &str = "nalgaeset.xml";

/// `$XDG_CONFIG_HOME` (비었으면 `$HOME/.config`). 둘 다 없으면 None.
pub fn xdg_config_home() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })
}

/// presguel 사용자 설정 디렉터리(`$XDG_CONFIG_HOME/presguel`).
pub fn presguel_config_dir() -> Option<PathBuf> {
    Some(xdg_config_home()?.join(APP_DIR))
}

/// 시스템 데이터 디렉터리 목록(`$XDG_DATA_DIRS`, 비었으면 `/usr/local/share:/usr/share`).
fn xdg_data_dirs() -> Vec<PathBuf> {
    std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string())
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// 자판(입력 설정) 파일 경로를 해석한다. 실제로 존재하는 첫 후보를 돌려준다
/// (`$PRESGUEL_CONFIG` 는 존재 여부와 무관하게 그대로 사용). 탐색 순서:
///
/// 1. `$PRESGUEL_CONFIG` (명시 지정)
/// 2. `$XDG_CONFIG_HOME/presguel/layout.xml` (사용자)
/// 3. `$XDG_CONFIG_HOME/presguel/nalgaeset.xml` (사용자, 옛 이름·하위 호환)
/// 4. `<XDG_DATA_DIRS>/presguel/layout.xml` (시스템 기본값)
///
/// 모두 없으면 어디를 찾았는지 안내하는 에러.
pub fn resolve_layout_path() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("PRESGUEL_CONFIG") {
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
        }
    }
    let mut tried: Vec<PathBuf> = Vec::new();
    if let Some(dir) = presguel_config_dir() {
        for name in [LAYOUT_FILE, LAYOUT_FILE_LEGACY] {
            let p = dir.join(name);
            if p.is_file() {
                return Ok(p);
            }
            tried.push(p);
        }
    }
    for d in xdg_data_dirs() {
        let p = d.join(APP_DIR).join(LAYOUT_FILE);
        if p.is_file() {
            return Ok(p);
        }
        tried.push(p);
    }
    let list = tried
        .iter()
        .map(|p| format!("  {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "자판 파일을 찾지 못했습니다. 다음 위치를 확인했습니다:\n{list}\n\
         PRESGUEL_CONFIG 환경변수로 경로를 지정하거나 위 중 한 곳에 두세요."
    ))
}
