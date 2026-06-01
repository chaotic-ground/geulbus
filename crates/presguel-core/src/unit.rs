//! 날개셋 낱자(단위) 모델과 니모닉/operand 해석.
//!
//! `H3|<operand>` 의 operand 는 니모닉(`_GG`, `O_`, `RS`)이거나 숫자(`0x1F4`,
//! `0x810000`)다. 니모닉은 위치(초/중/종)와 글자 정체를, 숫자는 갈마들이 토글(500)이나
//! 가상 단위(`id<<16`)를 나타낸다. 참고: `research/01-nalgaeset-format.md` §2,§7.
//!
//! 한글 자모 유니코드 사실(조합/분해, 호환 자모 다리)은 `hanmo` 크레이트에 있다.

/// 자모의 위치(낱자 갈래).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Category {
    /// 초성.
    Cho,
    /// 중성.
    Jung,
    /// 종성(받침).
    Jong,
}

/// 해결된 한글 자모 단위: 위치 + 조합용 자모 코드포인트.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Jamo {
    pub category: Category,
    /// 조합용 자모 코드포인트(U+1100 영역, 또는 옛한글/확장 영역).
    pub cp: u32,
}

impl Jamo {
    pub fn new(category: Category, cp: u32) -> Self {
        Self { category, cp }
    }

    /// 현대 자모 집합에 속하면 그 호환 자모(U+31xx)를 돌려준다. 옛한글이면 `None`
    /// (이 경우 설정의 FinalConvTable 에 의존).
    pub fn default_compat(&self) -> Option<u32> {
        match self.category {
            Category::Cho => hanmo::cho_compat(self.cp),
            Category::Jung => hanmo::jung_compat(self.cp),
            Category::Jong => hanmo::jong_compat(self.cp),
        }
    }
}

/// `H3|<operand>` 가 가리키는 단위.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    /// 보통의 한글 자모.
    Jamo(Jamo),
    /// 갈마들이 같은-키 토글 sentinel (500 = 0x1F4).
    Toggle,
    /// 미해결 가상 단위 id (VirtualUnitTable 로 해결해야 함). 예: 128/129/130.
    Virtual(u32),
}

/// 갈마들이 토글을 나타내는 내부 단위 값.
pub const TOGGLE: u32 = 500;

/// 니모닉의 "핵심 토큰"(밑줄 제거) → 호환 자모(글자 정체). 위치 무관.
fn mnemonic_to_compat(core: &str) -> Option<u32> {
    Some(match core {
        // 자음 (단일)
        "G" => 0x3131,
        "N" => 0x3134,
        "D" => 0x3137,
        "R" | "L" => 0x3139,
        "M" => 0x3141,
        "B" => 0x3142,
        "S" => 0x3145,
        "Q" | "NG" => 0x3147,
        "J" => 0x3148,
        "C" => 0x314A,
        "K" => 0x314B,
        "T" => 0x314C,
        "P" => 0x314D,
        "H" => 0x314E,
        // 자음 (쌍/겹)
        "GG" => 0x3132,
        "GS" => 0x3133,
        "NJ" => 0x3135,
        "NH" => 0x3136,
        "DD" => 0x3138,
        "RG" => 0x313A,
        "RM" => 0x313B,
        "RB" => 0x313C,
        "RS" => 0x313D,
        "RT" => 0x313E,
        "RP" => 0x313F,
        "RH" => 0x3140,
        "BB" => 0x3143,
        "BS" => 0x3144,
        "SS" => 0x3146,
        "JJ" => 0x3149,
        // 모음
        "A" => 0x314F,
        "AE" => 0x3150,
        "YA" => 0x3151,
        "YAE" => 0x3152,
        "EO" => 0x3153,
        "E" => 0x3154,
        "YEO" => 0x3155,
        "YE" => 0x3156,
        "O" => 0x3157,
        "WA" => 0x3158,
        "WAE" => 0x3159,
        "OI" => 0x315A,
        "YO" => 0x315B,
        "U" => 0x315C,
        "UEO" => 0x315D,
        "WE" => 0x315E,
        "WI" => 0x315F,
        "YU" => 0x3160,
        "EU" => 0x3161,
        "EUI" => 0x3162,
        "I" => 0x3163,
        _ => return None,
    })
}

/// 니모닉을 단위로 해석. `ctx` 가 주어지면(UnitMix 처럼) 그 위치를 강제하고, 없으면
/// 밑줄 위치(`_X`=종성, `X_`=초성/모음)와 글자 정체로 추론한다.
pub fn resolve_mnemonic(s: &str, ctx: Option<Category>) -> Option<Unit> {
    let (core, pos_category): (&str, Option<Category>) = if let Some(rest) = s.strip_prefix('_') {
        (rest, Some(Category::Jong))
    } else if let Some(rest) = s.strip_suffix('_') {
        (rest, None) // 초성 또는 모음 (글자 정체로 결정)
    } else {
        (s, None)
    };
    let compat = mnemonic_to_compat(core)?;
    let category = ctx.or(pos_category).unwrap_or_else(|| {
        if hanmo::is_vowel_compat(compat) {
            Category::Jung
        } else {
            Category::Cho
        }
    });
    let cp = match category {
        Category::Cho => hanmo::cho_cp_for_compat(compat)?,
        Category::Jung => hanmo::jung_cp_for_compat(compat)?,
        Category::Jong => hanmo::jong_cp_for_compat(compat)?,
    };
    Some(Unit::Jamo(Jamo::new(category, cp)))
}

/// 숫자 operand 를 단위로 해석.
/// - 500 → 갈마들이 토글.
/// - `id<<16` (하위 16비트 0, 상위 비0) → 가상 단위 id.
/// - 그 외 → 조합용 자모 코드포인트로 보고 영역으로 위치 추론(옛한글 포함).
pub fn resolve_numeric(n: u32) -> Option<Unit> {
    if n == TOGGLE {
        return Some(Unit::Toggle);
    }
    if n & 0xFFFF == 0 && n >> 16 != 0 {
        return Some(Unit::Virtual(n >> 16));
    }
    category_of_codepoint(n).map(|cat| Unit::Jamo(Jamo::new(cat, n)))
}

/// 조합용 자모 코드포인트의 위치를 블록 범위로 추론(옛한글/확장 포함).
pub fn category_of_codepoint(cp: u32) -> Option<Category> {
    match cp {
        0x1100..=0x115F => Some(Category::Cho), // 초성(현대+옛) + 초성 채움
        0x1160..=0x11A7 => Some(Category::Jung), // 중성 채움 + 중성(현대+옛)
        0x11A8..=0x11FF => Some(Category::Jong), // 종성(현대+옛)
        0xA960..=0xA97F => Some(Category::Cho), // 확장-A: 옛 초성
        0xD7B0..=0xD7CA => Some(Category::Jung), // 확장-B: 옛 중성
        0xD7CB..=0xD7FF => Some(Category::Jong), // 확장-B: 옛 종성
        _ => None,
    }
}

/// `H3|<operand>` operand 문자열(니모닉 또는 숫자)을 단위로 해석.
pub fn resolve_operand(s: &str, ctx: Option<Category>) -> Option<Unit> {
    let s = s.trim();
    if let Some(n) = parse_int(s) {
        resolve_numeric(n)
    } else {
        resolve_mnemonic(s, ctx)
    }
}

/// `0x..` 16진 또는 10진 정수 파싱.
pub fn parse_int(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jamo_of(u: Unit) -> Jamo {
        match u {
            Unit::Jamo(j) => j,
            other => panic!("expected jamo, got {other:?}"),
        }
    }

    #[test]
    fn keytable_mnemonics() {
        // 초성: k=G_ → ㄱ 초성 U+1100
        assert_eq!(
            jamo_of(resolve_mnemonic("G_", None).unwrap()),
            Jamo::new(Category::Cho, 0x1100)
        );
        // 종성: x=_G → ㄱ 종성 U+11A8
        assert_eq!(
            jamo_of(resolve_mnemonic("_G", None).unwrap()),
            Jamo::new(Category::Jong, 0x11A8)
        );
        // 중성: f=A_ → ㅏ U+1161, / = O_ → ㅗ U+1169
        assert_eq!(
            jamo_of(resolve_mnemonic("A_", None).unwrap()),
            Jamo::new(Category::Jung, 0x1161)
        );
        assert_eq!(
            jamo_of(resolve_mnemonic("O_", None).unwrap()),
            Jamo::new(Category::Jung, 0x1169)
        );
        // 바른 중성 bare: 8=EUI → ㅢ U+1174
        assert_eq!(
            jamo_of(resolve_mnemonic("EUI", None).unwrap()),
            Jamo::new(Category::Jung, 0x1174)
        );
        // 겹받침 종성: @=_RG → ㄺ U+11B0
        assert_eq!(
            jamo_of(resolve_mnemonic("_RG", None).unwrap()),
            Jamo::new(Category::Jong, 0x11B0)
        );
        // 초성 ㅇ: j=Q_ → U+110B
        assert_eq!(
            jamo_of(resolve_mnemonic("Q_", None).unwrap()),
            Jamo::new(Category::Cho, 0x110B)
        );
        // 종성 ㅇ: a=_Q → U+11BC
        assert_eq!(
            jamo_of(resolve_mnemonic("_Q", None).unwrap()),
            Jamo::new(Category::Jong, 0x11BC)
        );
    }

    #[test]
    fn unitmix_context_mnemonics() {
        // UnitMix JONG: R_ + S_ → RS, 모두 종성 ctx
        assert_eq!(
            jamo_of(resolve_mnemonic("R_", Some(Category::Jong)).unwrap()),
            Jamo::new(Category::Jong, 0x11AF) // ㄹ 종성
        );
        assert_eq!(
            jamo_of(resolve_mnemonic("S_", Some(Category::Jong)).unwrap()),
            Jamo::new(Category::Jong, 0x11BA) // ㅅ 종성
        );
        assert_eq!(
            jamo_of(resolve_mnemonic("RS", Some(Category::Jong)).unwrap()),
            Jamo::new(Category::Jong, 0x11B3) // ㄽ
        );
        // UnitMix CHO: GG → ㄲ 초성 U+1101
        assert_eq!(
            jamo_of(resolve_mnemonic("GG", Some(Category::Cho)).unwrap()),
            Jamo::new(Category::Cho, 0x1101)
        );
        // UnitMix JUNG: WA → ㅘ U+116A
        assert_eq!(
            jamo_of(resolve_mnemonic("WA", Some(Category::Jung)).unwrap()),
            Jamo::new(Category::Jung, 0x116A)
        );
    }

    #[test]
    fn numeric_operands() {
        // 0x1F4 = 500 = 갈마들이 토글
        assert_eq!(resolve_operand("0x1F4", None), Some(Unit::Toggle));
        assert_eq!(resolve_operand("500", None), Some(Unit::Toggle));
        // 0x800000 = 128<<16 = 가상 단위 128, 0x810000=129, 0x820000=130
        assert_eq!(resolve_operand("0x800000", None), Some(Unit::Virtual(128)));
        assert_eq!(resolve_operand("0x810000", None), Some(Unit::Virtual(129)));
        assert_eq!(resolve_operand("0x820000", None), Some(Unit::Virtual(130)));
    }

    #[test]
    fn raw_old_hangul_codepoint() {
        // 옛이응 초성 U+114C → 초성으로 분류
        assert_eq!(
            resolve_operand("0x114C", None),
            Some(Unit::Jamo(Jamo::new(Category::Cho, 0x114C)))
        );
        // 아래아 중성 U+119E → 중성
        assert_eq!(
            resolve_operand("0x119E", None),
            Some(Unit::Jamo(Jamo::new(Category::Jung, 0x119E)))
        );
    }

    #[test]
    fn default_compat_roundtrip() {
        assert_eq!(
            Jamo::new(Category::Cho, 0x1100).default_compat(),
            Some(0x3131)
        );
        assert_eq!(
            Jamo::new(Category::Jong, 0x11A8).default_compat(),
            Some(0x3131)
        );
        assert_eq!(
            Jamo::new(Category::Jung, 0x1161).default_compat(),
            Some(0x314F)
        );
        assert_eq!(
            Jamo::new(Category::Jong, 0x11B0).default_compat(),
            Some(0x313A)
        );
    }
}
