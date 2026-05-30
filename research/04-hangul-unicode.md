# Hangul Jamo Unicode Reference (composition + FinalConv)

Authoritative reference for implementing Hangul composition (modern + old / 옛한글 첫가끝) and
the `FinalConv` standalone-jamo output (conjoining jamo -> compatibility jamo).

Scope and verified facts:
- Conjoining (첫가끝) jamo live in **Hangul Jamo** `U+1100..U+11FF`, **Hangul Jamo Extended-A**
  `U+A960..U+A97F` (archaic *choseong*), and **Hangul Jamo Extended-B** `U+D7B0..U+D7FF`
  (archaic *jungseong* + *jongseong* positional forms). Extended blocks added in Unicode 5.2.
- Compatibility jamo live in **Hangul Compatibility Jamo** `U+3130..U+318F`. They have **no
  conjoining semantics and do NOT distinguish 초성 vs 종성** (one letter per shape).
- The `FinalConvTable` in `nalgaeset.xml` maps conjoining/PUA jamo -> compatibility jamo for
  rendering a *standalone* (uncombined) jamo. It is the authoritative mapping for our engine.

Sources: Unicode charts `n_3130`, Wikipedia *List of Hangul jamo* / *Hangul Compatibility Jamo* /
*Hangul Jamo (Unicode block)*, Unicode Korean FAQ. 한양 PUA range from 날개셋 docs / 위키백과.

---

## 1. CONJOINING JAMO (첫가끝) — modern indexed sets

### 1a. Modern 초성 (leading), choIdx 0..18 = `U+1100..U+1112` (19)

| idx | cp     | char | name (HANGUL CHOSEONG ...) |
|-----|--------|------|----------------------------|
| 0   | U+1100 | ᄀ   | KIYEOK |
| 1   | U+1101 | ᄁ   | SSANGKIYEOK |
| 2   | U+1102 | ᄂ   | NIEUN |
| 3   | U+1103 | ᄃ   | TIKEUT |
| 4   | U+1104 | ᄄ   | SSANGTIKEUT |
| 5   | U+1105 | ᄅ   | RIEUL |
| 6   | U+1106 | ᄆ   | MIEUM |
| 7   | U+1107 | ᄇ   | PIEUP |
| 8   | U+1108 | ᄈ   | SSANGPIEUP |
| 9   | U+1109 | ᄉ   | SIOS |
| 10  | U+110A | ᄊ   | SSANGSIOS |
| 11  | U+110B | ᄋ   | IEUNG |
| 12  | U+110C | ᄌ   | CIEUC |
| 13  | U+110D | ᄍ   | SSANGCIEUC |
| 14  | U+110E | ᄎ   | CHIEUCH |
| 15  | U+110F | ᄏ   | KHIEUKH |
| 16  | U+1110 | ᄐ   | THIEUTH |
| 17  | U+1111 | ᄑ   | PHIEUPH |
| 18  | U+1112 | ᄒ   | HIEUH |

### 1b. Modern 중성 (vowel), jungIdx 0..20 = `U+1161..U+1175` (21)

| idx | cp     | char | name (HANGUL JUNGSEONG ...) |
|-----|--------|------|-----------------------------|
| 0   | U+1161 | ᅡ   | A |
| 1   | U+1162 | ᅢ   | AE |
| 2   | U+1163 | ᅣ   | YA |
| 3   | U+1164 | ᅤ   | YAE |
| 4   | U+1165 | ᅥ   | EO |
| 5   | U+1166 | ᅦ   | E |
| 6   | U+1167 | ᅧ   | YEO |
| 7   | U+1168 | ᅨ   | YE |
| 8   | U+1169 | ᅩ   | O |
| 9   | U+116A | ᅪ   | WA |
| 10  | U+116B | ᅫ   | WAE |
| 11  | U+116C | ᅬ   | OE |
| 12  | U+116D | ᅭ   | YO |
| 13  | U+116E | ᅮ   | U |
| 14  | U+116F | ᅯ   | WEO |
| 15  | U+1170 | ᅰ   | WE |
| 16  | U+1171 | ᅱ   | WI |
| 17  | U+1172 | ᅲ   | YU |
| 18  | U+1173 | ᅳ   | EU |
| 19  | U+1174 | ᅴ   | YI |
| 20  | U+1175 | ᅵ   | I |

### 1c. Modern 종성 (trailing), jongIdx 1..27 = `U+11A8..U+11C2` (27) + slot 0 = no jongseong

`jongIdx 0` = no trailing (the "filler" slot in the formula; **no codepoint** is emitted for it).

| jongIdx | cp     | char | name (HANGUL JONGSEONG ...) |
|---------|--------|------|-----------------------------|
| 0       | —      | —    | (none / no jongseong)       |
| 1       | U+11A8 | ᆨ   | KIYEOK |
| 2       | U+11A9 | ᆩ   | SSANGKIYEOK |
| 3       | U+11AA | ᆪ   | KIYEOK-SIOS |
| 4       | U+11AB | ᆫ   | NIEUN |
| 5       | U+11AC | ᆬ   | NIEUN-CIEUC |
| 6       | U+11AD | ᆭ   | NIEUN-HIEUH |
| 7       | U+11AE | ᆮ   | TIKEUT |
| 8       | U+11AF | ᆯ   | RIEUL |
| 9       | U+11B0 | ᆰ   | RIEUL-KIYEOK |
| 10      | U+11B1 | ᆱ   | RIEUL-MIEUM |
| 11      | U+11B2 | ᆲ   | RIEUL-PIEUP |
| 12      | U+11B3 | ᆳ   | RIEUL-SIOS |
| 13      | U+11B4 | ᆴ   | RIEUL-THIEUTH |
| 14      | U+11B5 | ᆵ   | RIEUL-PHIEUPH |
| 15      | U+11B6 | ᆶ   | RIEUL-HIEUH |
| 16      | U+11B7 | ᆷ   | MIEUM |
| 17      | U+11B8 | ᆸ   | PIEUP |
| 18      | U+11B9 | ᆹ   | PIEUP-SIOS |
| 19      | U+11BA | ᆺ   | SIOS |
| 20      | U+11BB | ᆻ   | SSANGSIOS |
| 21      | U+11BC | ᆼ   | IEUNG |
| 22      | U+11BD | ᆽ   | CIEUC |
| 23      | U+11BE | ᆾ   | CHIEUCH |
| 24      | U+11BF | ᆿ   | KHIEUKH |
| 25      | U+11C0 | ᇀ   | THIEUTH |
| 26      | U+11C1 | ᇁ   | PHIEUPH |
| 27      | U+11C2 | ᇂ   | HIEUH |

### 1d. Internal layout of the Hangul Jamo block `U+1100..U+11FF`

| sub-range        | role |
|------------------|------|
| `U+1100..U+1112` | modern choseong (19) |
| `U+1113..U+115E` | old/archaic choseong |
| `U+115F`         | **CHOSEONG FILLER** |
| `U+1160`         | **JUNGSEONG FILLER** |
| `U+1161..U+1175` | modern jungseong (21) |
| `U+1176..U+11A7` | old/archaic jungseong |
| `U+11A8..U+11C2` | modern jongseong (27) |
| `U+11C3..U+11FF` | old/archaic jongseong |

Extended-A `U+A960..U+A97F`: **archaic choseong** only (composes as a leading jamo).
Extended-B `U+D7B0..U+D7CB`: **archaic jungseong**; `U+D7CB..U+D7FB`: **archaic jongseong**
(positional forms used for dynamic composition of syllables with no precomposed form).

---

## 2. PRECOMPOSITION FORMULA

```
S = SBase + (choIdx * VCount + jungIdx) * TCount + jongIdx
  = 0xAC00 + (choIdx * 21 + jungIdx) * 28 + jongIdx
```

Constants (Unicode UAX #15 / §3.12):

```
SBase  = 0xAC00   // first precomposed syllable (가)
LBase  = 0x1100   // first choseong
VBase  = 0x1161   // first jungseong
TBase  = 0x11A7   // == U+11A8 - 1  (so jongIdx 0 = "no trailing")
LCount = 19
VCount = 21
TCount = 28       // 27 trailing + 1 "none"
NCount = 588      // VCount * TCount
SCount = 11172    // LCount * NCount  (U+AC00..U+D7A3)
```

Valid **only** when choIdx∈0..18 and jungIdx∈0..20 from the modern indexed sets; jongIdx∈0..27
(0 optional). Any jamo outside the modern sets (old jamo, Extended-A/B, PUA) has **no precomposed
syllable** and must render via 첫가끝 conjoining sequences (needs an OpenType old-hangul font).

### Conjoining codepoint -> index (forward)

```
choIdx = cp - 0x1100   // valid 0..18 when 0x1100 <= cp <= 0x1112
jungIdx = cp - 0x1161  // valid 0..20 when 0x1161 <= cp <= 0x1175
jongIdx = cp - 0x11A7  // valid 1..27 when 0x11A8 <= cp <= 0x11C2 ; 0 = absent
```

### Index -> conjoining codepoint (reverse)

```
cho cp  = 0x1100 + choIdx
jung cp = 0x1161 + jungIdx
jong cp = (jongIdx == 0) ? none : 0x11A7 + jongIdx
```

### Decomposition of a precomposed syllable S (0xAC00..0xD7A3)

```
SIndex  = S - 0xAC00
choIdx  = SIndex / 588            // 588 = NCount
jungIdx = (SIndex % 588) / 28
jongIdx =  SIndex % 28            // 0 => no trailing
```

### Rust-friendly arrays (index order)

```rust
// choIdx 0..=18  -> conjoining choseong codepoint
pub const CHO: [u32; 19] = [
    0x1100, 0x1101, 0x1102, 0x1103, 0x1104, 0x1105, 0x1106, 0x1107, 0x1108, 0x1109,
    0x110A, 0x110B, 0x110C, 0x110D, 0x110E, 0x110F, 0x1110, 0x1111, 0x1112,
];

// jungIdx 0..=20 -> conjoining jungseong codepoint
pub const JUNG: [u32; 21] = [
    0x1161, 0x1162, 0x1163, 0x1164, 0x1165, 0x1166, 0x1167, 0x1168, 0x1169, 0x116A,
    0x116B, 0x116C, 0x116D, 0x116E, 0x116F, 0x1170, 0x1171, 0x1172, 0x1173, 0x1174,
    0x1175,
];

// jongIdx 0..=27 -> conjoining jongseong codepoint (index 0 = no trailing => 0)
pub const JONG: [u32; 28] = [
    0x0000, // 0 = none
    0x11A8, 0x11A9, 0x11AA, 0x11AB, 0x11AC, 0x11AD, 0x11AE, 0x11AF, 0x11B0, 0x11B1,
    0x11B2, 0x11B3, 0x11B4, 0x11B5, 0x11B6, 0x11B7, 0x11B8, 0x11B9, 0x11BA, 0x11BB,
    0x11BC, 0x11BD, 0x11BE, 0x11BF, 0x11C0, 0x11C1, 0x11C2,
];
```

---

## 3. COMPATIBILITY JAMO `U+3130..U+318F`

No conjoining semantics; **one letter per shape** (초성 ㄱ and 종성 ㄱ both map to `U+3131`).
This is exactly why FinalConv targets this block for standalone display. `U+3130` and `U+318F`
are unassigned; `U+3164` is the HANGUL FILLER.

### Full block (assigned)

| cp     | char | name (HANGUL LETTER / FILLER) |
|--------|------|-------------------------------|
| U+3131 | ㄱ | KIYEOK |
| U+3132 | ㄲ | SSANGKIYEOK |
| U+3133 | ㄳ | KIYEOK-SIOS |
| U+3134 | ㄴ | NIEUN |
| U+3135 | ㄵ | NIEUN-CIEUC |
| U+3136 | ㄶ | NIEUN-HIEUH |
| U+3137 | ㄷ | TIKEUT |
| U+3138 | ㄸ | SSANGTIKEUT |
| U+3139 | ㄹ | RIEUL |
| U+313A | ㄺ | RIEUL-KIYEOK |
| U+313B | ㄻ | RIEUL-MIEUM |
| U+313C | ㄼ | RIEUL-PIEUP |
| U+313D | ㄽ | RIEUL-SIOS |
| U+313E | ㄾ | RIEUL-THIEUTH |
| U+313F | ㄿ | RIEUL-PHIEUPH |
| U+3140 | ㅀ | RIEUL-HIEUH |
| U+3141 | ㅁ | MIEUM |
| U+3142 | ㅂ | PIEUP |
| U+3143 | ㅃ | SSANGPIEUP |
| U+3144 | ㅄ | PIEUP-SIOS |
| U+3145 | ㅅ | SIOS |
| U+3146 | ㅆ | SSANGSIOS |
| U+3147 | ㅇ | IEUNG |
| U+3148 | ㅈ | CIEUC |
| U+3149 | ㅉ | SSANGCIEUC |
| U+314A | ㅊ | CHIEUCH |
| U+314B | ㅋ | KHIEUKH |
| U+314C | ㅌ | THIEUTH |
| U+314D | ㅍ | PHIEUPH |
| U+314E | ㅎ | HIEUH |
| U+314F | ㅏ | A |
| U+3150 | ㅐ | AE |
| U+3151 | ㅑ | YA |
| U+3152 | ㅒ | YAE |
| U+3153 | ㅓ | EO |
| U+3154 | ㅔ | E |
| U+3155 | ㅕ | YEO |
| U+3156 | ㅖ | YE |
| U+3157 | ㅗ | O |
| U+3158 | ㅘ | WA |
| U+3159 | ㅙ | WAE |
| U+315A | ㅚ | OE |
| U+315B | ㅛ | YO |
| U+315C | ㅜ | U |
| U+315D | ㅝ | WEO |
| U+315E | ㅞ | WE |
| U+315F | ㅟ | WI |
| U+3160 | ㅠ | YU |
| U+3161 | ㅡ | EU |
| U+3162 | ㅢ | YI (UI) |
| U+3163 | ㅣ | I |
| U+3164 | ㅤ | **HANGUL FILLER** |
| U+3165 | ㅥ | SSANGNIEUN |
| U+3166 | ㅦ | NIEUN-TIKEUT |
| U+3167 | ㅧ | NIEUN-SIOS |
| U+3168 | ㅨ | NIEUN-PANSIOS |
| U+3169 | ㅩ | RIEUL-KIYEOK-SIOS |
| U+316A | ㅪ | RIEUL-TIKEUT |
| U+316B | ㅫ | RIEUL-PIEUP-SIOS |
| U+316C | ㅬ | RIEUL-PANSIOS |
| U+316D | ㅭ | RIEUL-YEORINHIEUH |
| U+316E | ㅮ | MIEUM-PIEUP |
| U+316F | ㅯ | MIEUM-SIOS |
| U+3170 | ㅰ | MIEUM-PANSIOS |
| U+3171 | ㅱ | KAPYEOUNMIEUM (순경음 ㅁ) |
| U+3172 | ㅲ | PIEUP-KIYEOK |
| U+3173 | ㅳ | PIEUP-TIKEUT |
| U+3174 | ㅴ | PIEUP-SIOS-KIYEOK |
| U+3175 | ㅵ | PIEUP-SIOS-TIKEUT |
| U+3176 | ㅶ | PIEUP-CIEUC |
| U+3177 | ㅷ | PIEUP-THIEUTH |
| U+3178 | ㅸ | KAPYEOUNPIEUP (순경음 ㅂ) |
| U+3179 | ㅹ | KAPYEOUNSSANGPIEUP |
| U+317A | ㅺ | SIOS-KIYEOK |
| U+317B | ㅻ | SIOS-NIEUN |
| U+317C | ㅼ | SIOS-TIKEUT |
| U+317D | ㅽ | SIOS-PIEUP |
| U+317E | ㅾ | SIOS-CIEUC |
| U+317F | ㅿ | PANSIOS (반시옷 ㅿ) |
| U+3180 | ㆀ | SSANGIEUNG |
| U+3181 | ㆁ | YESIEUNG (옛이응 ㆁ) |
| U+3182 | ㆂ | YESIEUNG-SIOS |
| U+3183 | ㆃ | YESIEUNG-PANSIOS |
| U+3184 | ㆄ | KAPYEOUNPHIEUPH (순경음 ㅍ) |
| U+3185 | ㆅ | SSANGHIEUH |
| U+3186 | ㆆ | YEORINHIEUH (여린히읗 ㆆ) |
| U+3187 | ㆇ | YO-YA |
| U+3188 | ㆈ | YO-YAE |
| U+3189 | ㆉ | YO-I |
| U+318A | ㆊ | YU-YEO |
| U+318B | ㆋ | YU-YE |
| U+318C | ㆌ | YU-I |
| U+318D | ㆍ | ARAEA (아래아 ㆍ) |
| U+318E | ㆎ | ARAEAE |

### FinalConvTable cross-check (from `nalgaeset.xml`)

| from (conjoining) | to (compat) | check |
|-------------------|-------------|-------|
| `0x1100` CHOSEONG KIYEOK | `0x3131` LETTER KIYEOK | OK |
| `0x11A8` JONGSEONG KIYEOK | `0x3131` LETTER KIYEOK | OK — same target as choseong, confirms no 초/종 distinction |
| `0x1161` JUNGSEONG A | `0x314F` LETTER A | OK |
| `0x11AA` JONGSEONG KIYEOK-SIOS | `0x3133` LETTER KIYEOK-SIOS | OK |
| `0x116A` JUNGSEONG WA | `0x3158` LETTER WA | OK |

The table is one-directional (conjoining/PUA -> compat) and **lossy**: many conjoining cps fold to
one compat cp, so it is not reversible to recover 초성/종성.

---

## 4. HALF-WIDTH / FILLERS

| cp     | name | role in 첫가끝 |
|--------|------|----------------|
| U+115F | HANGUL CHOSEONG FILLER  | placeholder for a **missing leading** slot in a syllable block (vowel-only / V-T block) |
| U+1160 | HANGUL JUNGSEONG FILLER | placeholder for a **missing vowel** slot (L-only / L-T block) |
| U+3164 | HANGUL FILLER (compat)  | legacy KS X 1001 stand-in for an absent element; **not** a conjoining filler |

In 첫가끝, an OpenType old-hangul font shapes a *complete* L+V(+T) sequence into one block. To show a
syllable block that is *missing* a slot (e.g. a lone 종성 under a square, or a vowel with no leading
consonant), you insert the matching conjoining filler so the shaper still sees a well-formed
L+V(+T) sequence:
- missing leading: `U+115F` + jungseong (+ jongseong)
- missing vowel: choseong + `U+1160` (+ jongseong)

`U+3164` (compat filler) belongs to the compatibility block and is used in legacy plain-text
contexts, not in conjoining sequences.

### Recommendation for our engine

For an **incomplete syllable that the user is composing standalone** (a lone jamo, no full L+V+T),
prefer **FinalConv -> compatibility jamo** (`U+3131..`) rather than emitting conjoining-jamo +
filler. Reasons:
- compat jamo render with *any* normal Korean font; conjoining + filler need an OT old-hangul font;
- this matches what `nalgaeset.xml` already does (a single lone jamo is output via FinalConv);
- emit conjoining sequences (optionally with `U+115F`/`U+1160`) only when the engine is in
  true 옛한글 mode and is producing a *renderable syllable block* (e.g. archaic L+V where no
  precomposed syllable exists), where an OT font is assumed.

So: **standalone single jamo -> FinalConv to compat; full/partial old syllable blocks -> conjoining
첫가끝 (use fillers for the missing L or V slot).**

---

## 5. OLD HANGUL (옛한글) notes

- **Rendering**: 첫가끝 (L + V + optional T, conjoining codepoints) is *not* precomposed; it relies
  on an OpenType font with Hangul shaping (GSUB/GPOS, `ljmo`/`vjmo`/`tjmo` features) to stack the
  jamo into one square. Without such a font the jamo render side-by-side. Modern L+V(+T) in the
  modern ranges should be normalized to precomposed `U+AC00..U+D7A3` via the formula in §2.
- **Extended blocks** in the FinalConvTable:
  - `U+A960..U+A97F` (Extended-A) = **archaic choseong** (e.g. `0xA964`, `0xA966`, `0xA968`,
    `0xA969`, `0xA96C`, `0xA971` in the table) — leading-position old jamo.
  - `U+D7B0..U+D7FF` (Extended-B) = **archaic jungseong + jongseong** positional forms (e.g.
    `0xD7CD`, `0xD7E3`, `0xD7E6..0xD7E8`, `0xD7EF`, `0xD7F9` in the table) — these are standard
    Unicode 5.2 old jamo, *not* PUA.
- **PUA codepoints** `0xEAxx` and `0xECxx` in the FinalConvTable are **한양 PUA (Hanyang PUA)** old
  jamo — a pre-Unicode-5.2 non-standard scheme (Hanyang PUA spans roughly `U+E0BC..U+EFFF` and
  `U+F100..U+F66E`). 날개셋 supports both 첫가끝 and Hanyang PUA and can convert between them.
  These cps are private-use, so their *meaning* is defined only by the font / the FinalConvTable;
  the table tells us each PUA jamo's standalone compatibility-jamo glyph (e.g. `0xEA07 -> 0x3133`
  ㄳ, `0xEA40 -> 0x313E` ㄾ, `0xEC57 -> 0x3172` ㅲ).
- **For our engine**: we do not need to interpret PUA/Extended semantics ourselves — the
  FinalConvTable is the authoritative source->standalone mapping. Treat any `from` cp simply as an
  opaque jamo whose standalone display form is the mapped `to` compat cp. (Confirmed: the table is
  exactly such a lookup.)

---

## 6. COMPOUND JAMO (UnitMix builds these)

### Compound 중성 (vowels) — built from two simple vowels

| compound | conjoining cp | jungIdx | parts (날개셋 UnitMix) | compat cp |
|----------|---------------|---------|------------------------|-----------|
| ㅘ WA    | U+116A | 9  | O_ + A_  (`O_`+`A_` -> `WA`)   | U+3158 |
| ㅙ WAE   | U+116B | 10 | O_ + AE  (`O_`+`AE` -> `WAE`)  | U+3159 |
| ㅚ OE    | U+116C | 11 | O_ + I_  (`O_`+`I_` -> `OI`)   | U+315A |
| ㅝ WEO   | U+116F | 14 | U_ + EO  (`U_`+`EO` -> `UEO`)  | U+315D |
| ㅞ WE    | U+1170 | 15 | U_ + E_  (`U_`+`E_` -> `WE`)   | U+315E |
| ㅟ WI    | U+1171 | 16 | U_ + I_  (`U_`+`I_` -> `WI`)   | U+315F |
| ㅢ YI/UI | U+1174 | 19 | EU + I_  (ㅡ + ㅣ)             | U+3162 |

(The `nalgaeset.xml` UnitMixTable defines WA/WAE/OI/UEO/WE/WI; ㅢ is formed analogously ㅡ+ㅣ.)

### Compound 종성 (trailing) — built from two simple finals

| compound | conjoining cp | jongIdx | parts | compat cp |
|----------|---------------|---------|-------|-----------|
| ㄳ KIYEOK-SIOS    | U+11AA | 3  | ㄱ + ㅅ | U+3133 |
| ㄵ NIEUN-CIEUC    | U+11AC | 5  | ㄴ + ㅈ | U+3135 |
| ㄶ NIEUN-HIEUH    | U+11AD | 6  | ㄴ + ㅎ | U+3136 |
| ㄺ RIEUL-KIYEOK   | U+11B0 | 9  | ㄹ + ㄱ | U+313A |
| ㄻ RIEUL-MIEUM    | U+11B1 | 10 | ㄹ + ㅁ | U+313B |
| ㄼ RIEUL-PIEUP    | U+11B2 | 11 | ㄹ + ㅂ (`R_`+`P_` -> `RP`) | U+313C |
| ㄽ RIEUL-SIOS     | U+11B3 | 12 | ㄹ + ㅅ (`R_`+`S_` -> `RS`) | U+313D |
| ㄾ RIEUL-THIEUTH  | U+11B4 | 13 | ㄹ + ㅌ (`R_`+`T_` -> `RT`) | U+313E |
| ㄿ RIEUL-PHIEUPH  | U+11B5 | 14 | ㄹ + ㅍ | U+313F |
| ㅀ RIEUL-HIEUH    | U+11B6 | 15 | ㄹ + ㅎ | U+3140 |
| ㅄ PIEUP-SIOS     | U+11B9 | 18 | ㅂ + ㅅ | U+3144 |

(`nalgaeset.xml` UnitMixTable explicitly defines the 종성 mixes RS=ㄽ, RT=ㄾ, RP=ㄼ; the others
follow the same simple+simple -> compound pattern.)

### Rust-friendly compound arrays

```rust
// (compound jungIdx, left jungIdx, right jungIdx) — vowel compounds
pub const JUNG_COMPOUND: [(u8, u8, u8); 7] = [
    (9, 8, 0),    // ㅘ = ㅗ(8) + ㅏ(0)
    (10, 8, 1),   // ㅙ = ㅗ(8) + ㅐ(1)
    (11, 8, 20),  // ㅚ = ㅗ(8) + ㅣ(20)
    (14, 13, 4),  // ㅝ = ㅜ(13) + ㅓ(4)
    (15, 13, 5),  // ㅞ = ㅜ(13) + ㅔ(5)
    (16, 13, 20), // ㅟ = ㅜ(13) + ㅣ(20)
    (19, 18, 20), // ㅢ = ㅡ(18) + ㅣ(20)
];

// (compound jongIdx, left jongIdx, right jongIdx) — final-consonant compounds
// jongIdx per §1c (1=ㄱ,4=ㄴ,8=ㄹ,17=ㅂ,19=ㅅ,22=ㅈ,25=ㅌ,26=ㅍ,27=ㅎ,16=ㅁ)
pub const JONG_COMPOUND: [(u8, u8, u8); 11] = [
    (3, 1, 19),   // ㄳ = ㄱ + ㅅ
    (5, 4, 22),   // ㄵ = ㄴ + ㅈ
    (6, 4, 27),   // ㄶ = ㄴ + ㅎ
    (9, 8, 1),    // ㄺ = ㄹ + ㄱ
    (10, 8, 16),  // ㄻ = ㄹ + ㅁ
    (11, 8, 17),  // ㄼ = ㄹ + ㅂ
    (12, 8, 19),  // ㄽ = ㄹ + ㅅ
    (13, 8, 25),  // ㄾ = ㄹ + ㅌ
    (14, 8, 26),  // ㄿ = ㄹ + ㅍ
    (15, 8, 27),  // ㅀ = ㄹ + ㅎ
    (18, 17, 19), // ㅄ = ㅂ + ㅅ
];
```
