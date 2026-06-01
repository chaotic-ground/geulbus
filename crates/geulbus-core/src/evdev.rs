//! Linux evdev keycode → 물리 키 위치의 US-QWERTY ASCII(shift 없는 기준) 매핑.
//!
//! 날개셋이 Windows 에서 스캔코드로 "물리 자리"를 잡았듯, geulbus 도 keycode 로 물리
//! 위치를 잡는다. keyval 은 사용자 XKB 레이아웃(드보락 등)에 따라 흔들리지만 keycode 는
//! 레이아웃과 무관하게 물리 위치를 가리키므로, KeyTable(물리 자리 기준)을 keycode 로
//! 조회하면 어떤 영문 배열에서도 세벌식 자리가 고정된다.
//!
//! 표는 표준 Linux evdev(`/usr/include/linux/input-event-codes.h`) 기준이며, 실기
//! 측정으로 교차검증했다(19=R, 23=I, 50=M, 20=T, 36=J, 3='2', 5='4').
//! 주의: IBus 의 keycode 는 "XKB keycode - 8 = evdev" 다(gnome-shell inputMethod.js
//! 가 `get_key_code() - 8` 로 evdev 를 넘김). 즉 여기 들어오는 값은 evdev 그대로다.

/// evdev keycode → 그 물리 키의 US-QWERTY 기본(비-shift) ASCII 문자.
/// KeyTable 인덱스(0x21..0x7E)와 같은 공간이다. 매핑이 없으면 `None`.
pub fn evdev_to_ascii(code: u32) -> Option<u8> {
    let c = match code {
        // 숫자열 KEY_1..KEY_0, KEY_MINUS, KEY_EQUAL (evdev 2..13)
        2 => b'1',
        3 => b'2',
        4 => b'3',
        5 => b'4',
        6 => b'5',
        7 => b'6',
        8 => b'7',
        9 => b'8',
        10 => b'9',
        11 => b'0',
        12 => b'-',
        13 => b'=',
        // 윗줄 KEY_Q..KEY_RIGHTBRACE (16..27)
        16 => b'q',
        17 => b'w',
        18 => b'e',
        19 => b'r',
        20 => b't',
        21 => b'y',
        22 => b'u',
        23 => b'i',
        24 => b'o',
        25 => b'p',
        26 => b'[',
        27 => b']',
        // 가운뎃줄 KEY_A..KEY_L, KEY_SEMICOLON, KEY_APOSTROPHE, KEY_GRAVE (30..41)
        30 => b'a',
        31 => b's',
        32 => b'd',
        33 => b'f',
        34 => b'g',
        35 => b'h',
        36 => b'j',
        37 => b'k',
        38 => b'l',
        39 => b';',
        40 => b'\'',
        41 => b'`',
        // 아랫줄 KEY_BACKSLASH, KEY_Z..KEY_SLASH (43..53)
        43 => b'\\',
        44 => b'z',
        45 => b'x',
        46 => b'c',
        47 => b'v',
        48 => b'b',
        49 => b'n',
        50 => b'm',
        51 => b',',
        52 => b'.',
        53 => b'/',
        _ => return None,
    };
    Some(c)
}

/// 기본 US-QWERTY ASCII 를 그 키의 **shift 누른** US-QWERTY 문자로 바꾼다.
/// (날개셋 KeyTable 은 shift 적용 후 문자로 인덱싱됨: 예 Shift+1='!', Shift+2='@'.)
pub fn us_shift(base: u8) -> u8 {
    match base {
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b'0' => b')',
        b'-' => b'_',
        b'=' => b'+',
        b'[' => b'{',
        b']' => b'}',
        b'\\' => b'|',
        b';' => b':',
        b'\'' => b'"',
        b'`' => b'~',
        b',' => b'<',
        b'.' => b'>',
        b'/' => b'?',
        b'a'..=b'z' => base - 0x20, // 대문자
        other => other,
    }
}

/// evdev keycode + shift → US-QWERTY 가 만들어낼 ASCII(shift 반영).
/// 이게 날개셋 KeyTable 의 인덱스 공간이다. 사용자 XKB 가 드보락이어도 같은 결과 →
/// 물리 위치 기준으로 세벌식이 고정된다. 매핑이 없으면 `None`.
pub fn us_qwerty_ascii(code: u32, shift: bool) -> Option<u8> {
    let base = evdev_to_ascii(code)?;
    Some(if shift { us_shift(base) } else { base })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifted_number_row_and_letters() {
        // 세벌식 shifted 자모의 전제: Shift+1='!', Shift+2='@' 등.
        assert_eq!(us_qwerty_ascii(2, true), Some(b'!')); // KEY_1
        assert_eq!(us_qwerty_ascii(3, true), Some(b'@')); // KEY_2
        assert_eq!(us_qwerty_ascii(19, true), Some(b'R')); // KEY_R 대문자
        assert_eq!(us_qwerty_ascii(19, false), Some(b'r'));
        assert_eq!(us_qwerty_ascii(40, true), Some(b'"')); // KEY_APOSTROPHE
        assert_eq!(us_qwerty_ascii(53, true), Some(b'?')); // KEY_SLASH
    }

    #[test]
    fn measured_pairs_match() {
        // 실기 측정에서 얻은 (evdev → QWERTY 문자) 쌍.
        assert_eq!(evdev_to_ascii(19), Some(b'r'));
        assert_eq!(evdev_to_ascii(23), Some(b'i'));
        assert_eq!(evdev_to_ascii(50), Some(b'm'));
        assert_eq!(evdev_to_ascii(20), Some(b't'));
        assert_eq!(evdev_to_ascii(36), Some(b'j'));
        assert_eq!(evdev_to_ascii(3), Some(b'2'));
        assert_eq!(evdev_to_ascii(5), Some(b'4'));
    }

    #[test]
    fn rows_are_complete() {
        // 세 글자줄이 끊김 없이 매핑되는지(대표 위치).
        assert_eq!(evdev_to_ascii(16), Some(b'q')); // 윗줄 시작
        assert_eq!(evdev_to_ascii(30), Some(b'a')); // 가운뎃줄 시작
        assert_eq!(evdev_to_ascii(44), Some(b'z')); // 아랫줄 시작
        assert_eq!(evdev_to_ascii(2), Some(b'1')); // 숫자열 시작
    }

    #[test]
    fn unmapped_returns_none() {
        assert_eq!(evdev_to_ascii(0), None);
        assert_eq!(evdev_to_ascii(1), None); // Esc
        assert_eq!(evdev_to_ascii(28), None); // Enter
        assert_eq!(evdev_to_ascii(57), None); // Space (별도 처리)
        assert_eq!(evdev_to_ascii(15), None); // Tab
    }

    #[test]
    fn all_mapped_are_in_keytable_range() {
        // 매핑된 모든 ASCII 가 KeyTable 인덱스 범위(0x21..=0x7E)에 든다.
        for code in 0..=120u32 {
            if let Some(a) = evdev_to_ascii(code) {
                assert!((0x21..=0x7e).contains(&a), "code {code} -> {a:#x} 범위 밖");
            }
        }
    }
}
