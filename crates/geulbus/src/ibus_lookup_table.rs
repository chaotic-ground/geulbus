//! IBusLookupTable 직렬화 — 후보창(한자·특수문자 변환)용.
//!
//! 엔진이 `UpdateLookupTable`(IBusLookupTable, visible) 신호를 보내면 패널이
//! 후보 목록을 띄운다. 후보 전체와 절대 cursor 위치를 담아 보내며, 페이지
//! 나누기(cursor/page_size)는 패널 몫이다.
//!
//! 직렬화 시그니처(research/03-ibus-zbus.md §3, ibus 1.5.x 소스 순서):
//! `IBusLookupTable : (sa{sv}uubbiavav)`  name, attach, page_size, cursor_pos,
//! cursor_visible, round, orientation, candidates(av of IBusText), labels(av).
//! 주의: candidates/labels 는 `av`(variant 배열)여야 한다 — `a(...)` 로 보내면
//! 데몬이 죽는다(ibus#2611).

use std::collections::HashMap;

use zbus::zvariant::{StructureBuilder, Value};

use crate::ibus_text::make_ibus_text;

/// 한 페이지 후보 수. MS IME/ibus-hangul 과 같은 9.
pub const PAGE_SIZE: u32 = 9;

/// 세로 방향(IBUS_ORIENTATION_VERTICAL). 훈음이 붙는 후보는 세로가 읽기 좋다.
const ORIENTATION_VERTICAL: i32 = 1;

/// 후보 문자열들로 IBusLookupTable 을 만든다: `(sa{sv}uubbiavav)`.
/// `cursor` 는 전체 목록 기준 절대 위치.
pub fn make_lookup_table(candidates: &[String], cursor: u32) -> Value<'static> {
    let items: Vec<Value<'static>> = candidates
        .iter()
        .map(|c| make_ibus_text(c.clone()))
        .collect();
    let s = StructureBuilder::new()
        .add_field("IBusLookupTable".to_string())
        .add_field(HashMap::<String, Value<'static>>::new())
        .add_field(PAGE_SIZE) // page_size (u)
        .add_field(cursor) // cursor_pos (u)
        .add_field(true) // cursor_visible (b)
        .add_field(true) // round (b): 엔진의 페이징이 순환(wrap)하므로 패널에도 알린다
        .add_field(ORIENTATION_VERTICAL) // orientation (i)
        .add_field(items) // candidates (av)
        .add_field(Vec::<Value<'static>>::new()) // labels (av, 비면 패널이 1..9 기본 번호)
        .build()
        .expect("valid ibus structure");
    Value::new(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_table_signature() {
        let v = make_lookup_table(&["學 배울 학".to_string(), "鶴 두루미 학".to_string()], 0);
        assert_eq!(v.value_signature().to_string(), "(sa{sv}uubbiavav)");
    }

    #[test]
    fn empty_table_signature_holds() {
        // 빈 av 라도 시그니처가 av 로 유지돼야 한다(a(...) 로 새면 데몬 사망, ibus#2611).
        let v = make_lookup_table(&[], 0);
        assert_eq!(v.value_signature().to_string(), "(sa{sv}uubbiavav)");
    }
}
