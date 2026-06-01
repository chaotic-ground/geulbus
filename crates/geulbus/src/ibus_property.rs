//! IBusProperty / IBusPropList 직렬화 — 패널 표시(한/영 심볼)용.
//!
//! 엔진이 `RegisterProperties`(IBusPropList)로 속성을 등록하고, 모드가 바뀔 때
//! `UpdateProperty`(IBusProperty)로 심볼을 갱신하면, GNOME/ibus 패널이 그 심볼을
//! 입력기 표시로 보여준다(ibus-hangul 의 한/EN 토글과 동일 방식).
//!
//! 직렬화 시그니처(ibus 1.5.x C 소스 순서 기준):
//! - `IBusProperty : (sa{sv}suvsvbbuvv)`  name, attach, key, type, label(IBusText),
//!   icon, tooltip(IBusText), sensitive, visible, state, sub_props(IBusPropList), symbol(IBusText)
//! - `IBusPropList : (sa{sv}av)`  name, attach, 속성 variant 배열

use std::collections::HashMap;

use zbus::zvariant::{StructureBuilder, Value};

use crate::ibus_text::make_ibus_text;

const PROP_TYPE_NORMAL: u32 = 0;
const PROP_STATE_UNCHECKED: u32 = 0;

fn empty_attach() -> HashMap<String, Value<'static>> {
    HashMap::new()
}

/// 빈 IBusPropList: `(sa{sv}av)`
fn empty_prop_list() -> Value<'static> {
    let s = StructureBuilder::new()
        .add_field("IBusPropList".to_string())
        .add_field(empty_attach())
        .add_field(Vec::<Value<'static>>::new())
        .build()
        .expect("valid ibus structure");
    Value::new(s)
}

/// 입력 모드 표시용 IBusProperty 하나: `(sa{sv}suvsvbbuvv)`.
/// `symbol` 이 패널에 보이는 글자(예: "한" / "EN").
pub fn make_input_mode_property(symbol: &str, label: &str) -> Value<'static> {
    let s = StructureBuilder::new()
        .add_field("IBusProperty".to_string())
        .add_field(empty_attach())
        .add_field("InputMode".to_string()) // key (s)
        .add_field(PROP_TYPE_NORMAL) // type (u)
        .add_field(make_ibus_text(label.to_string())) // label (v: IBusText)
        .add_field(String::new()) // icon (s)
        .add_field(make_ibus_text("")) // tooltip (v: IBusText)
        .add_field(true) // sensitive (b)
        .add_field(true) // visible (b)
        .add_field(PROP_STATE_UNCHECKED) // state (u)
        .add_field(empty_prop_list()) // sub_props (v: IBusPropList)
        .add_field(make_ibus_text(symbol.to_string())) // symbol (v: IBusText)
        .build()
        .expect("valid ibus structure");
    Value::new(s)
}

/// InputMode 속성 하나를 담은 IBusPropList: `(sa{sv}av)`.
pub fn make_prop_list(symbol: &str, label: &str) -> Value<'static> {
    let prop = make_input_mode_property(symbol, label);
    let s = StructureBuilder::new()
        .add_field("IBusPropList".to_string())
        .add_field(empty_attach())
        .add_field(vec![prop])
        .build()
        .expect("valid ibus structure");
    Value::new(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_signature() {
        let v = make_input_mode_property("한", "Input Mode");
        assert_eq!(v.value_signature().to_string(), "(sa{sv}suvsvbbuvv)");
    }

    #[test]
    fn prop_list_signature() {
        let v = make_prop_list("EN", "Input Mode");
        assert_eq!(v.value_signature().to_string(), "(sa{sv}av)");
    }
}
