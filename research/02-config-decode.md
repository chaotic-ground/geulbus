# 02 — 날개셋 config decode: `세벌식-맞춤` behavioral spec (test oracle)

Source file analyzed: `/home/nemo/git/lens/provision/config/nalgaeset.xml`
Target: `InputEntry[0]` = `KeyTable name="세벌식-맞춤"` (object `CBasicInputScheme`) +
`GeneratorSetting object="CNgsImeEx"`.

> This is a best-effort static reverse engineering of one 날개셋 (Nalgaeset) config.
> Unit mnemonics, conjoining-jamo codepoints and automaton semantics are derived from
> the file itself (especially the `FinalConvTable`, which is an authoritative
> conjoining→compatibility map embedded in this very config) plus standard Unicode jamo
> facts and the known 날개셋 mnemonic convention. Items I am unsure about are flagged
> inline with **(?)**.

---

## 0. Which entry is active / default

`<InputLayer default="0" current="2">`.

- There are **3** `InputEntry` elements (indices 0,1,2):
  - **`InputEntry[0]`** = `세벌식-맞춤` (`CBasicInputScheme` + `CNgsImeEx`). The Korean 3-set layout. **This is the one this doc specs.**
  - `InputEntry[1]` = `로마자 드보락` (`CAdvancedScheme` + `CNgsIme`). A Dvorak Latin layout with `^(P&1)<<5` case-shift expressions (P = shift state bit; `<<5` = ±0x20 case toggle). Not Korean.
  - `InputEntry[2]` = empty `CInputScheme` + `CIme` (pass-through / direct, i.e. "한글 끄기 / English direct").

- **`default="0"`** → the entry selected on a fresh start is **index 0 = `세벌식-맞춤`**.
- **`current="2"`** → at the moment this config was *saved*, the live/selected entry was **index 2** (the pass-through English/direct mode). This is just the persisted UI state; it does **not** mean entry 2 is the "real" layout. The Hangul layout under test is unambiguously **entry 0**.

So: **default = entry 0 (세벌식-맞춤), current(saved) = entry 2 (direct/pass-through).** The
spec subject is entry 0. `VK_HANGUL` / `VK_CAPITAL` are bound to `IME_SWITCH !A` (toggle
between Korean entry and the others).

---

## A. FULL KEY MAP — `세벌식-맞춤` KeyTable

### A.1 Notation legend

The `value` of each `<Key>` is a 날개셋 expression. Decoded forms:

| Form in value          | Meaning |
|------------------------|---------|
| `H3\|<unit>`           | emit a **Hangul jamo unit** into the automaton. `<unit>` is a unit mnemonic or a raw hex unit id. |
| `0xNN`                 | emit a **literal character** U+00NN (a symbol / digit / punctuation), commits/no-Hangul. |
| `T ? X : Y`            | **conditional**: `T` = "is a Hangul composition currently in progress?" If yes do `X`, else `Y`. (This is the 갈마들이 / context-shift mechanism: same physical key = jamo while composing, symbol otherwise.) |
| `C0\|N`                | a **control / command code** (C0 = command class 0). e.g. `C0\|0xA`, `C0\|2`. Editor actions (newline-ish, undo/redo, bracket-jump), not text jamo. |

**Unit mnemonic convention** (날개셋):
- `X_`  = **초성** (leading) form of consonant X. e.g. `G_` = 초성 ㄱ (U+1100).
- `_X`  = **종성** (trailing) form of consonant X. e.g. `_G` = 종성 ㄱ (U+11A8).
- bare vowel token (`A_`, `O_`, `YA`, `EUI`, `AE`, …) = **중성** (medial) vowel.
  (`A_`/`O_`/`U_`/`E_`/`I_` carry a trailing `_` but are vowels, not consonants — the `_`
  is just part of the single-letter-vowel mnemonic; `O_`, `U_`, `EU` also appear as
  `VirtualUnit`s, see §B.)
- Raw `H3|0xNNNNNN` (e.g. `0x800000`, `0x810000`, `0x820000`, `0x1F4`) = a unit referenced
  by **raw internal id** rather than mnemonic — these are the **filler/채움 units** and a
  couple of special jamo (decoded in §A.3).

`X` in mnemonics ↔ Latin consonant code: G=ㄱ N=ㄴ D=ㄷ R/L=ㄹ M=ㅁ B=ㅂ S=ㅅ NG/(O as 초성ㅇ? no)…
The consonant letter codes used here: **G ㄱ, N ㄴ, D ㄷ, R ㄹ, M ㅁ, B ㅂ, S ㅅ, Q ㅇ(초성 이응),
J ㅈ, C ㅊ, K ㅋ, T ㅌ, P ㅍ, H ㅎ**. (Note **Q = ㅇ** here, both as `Q_` 초성 ㅇ and `_Q` 종성 ㅇ.)
Cluster codes: `GG`=ㄲ `DD`=ㄸ `BB`=ㅃ `SS`=ㅆ `JJ`=ㅉ; jongseong clusters `GS`=ㄳ `NJ`=ㄵ `NH`=ㄶ
`RG`=ㄺ `RM`=ㄻ `RB`=ㄼ `RS`=ㄽ `RT`=ㄾ `RP`=ㄿ `RH`=ㅀ `BS`=ㅄ.

### A.2 The full table (at = 0x21..0x7E)

ASCII column = the **unshifted physical character on a US-QWERTY keycap** whose VK produces
this `at` code (날개셋 keys the table by the produced ASCII of the layout slot). Hand column:
초성=오른손(right), 중성=가운데(middle/index reach), 종성=왼손(left) per 세벌식 convention.

| at   | ASCII | value                       | Class | Decoded action |
|------|-------|-----------------------------|-------|----------------|
| 0x21 | `!`   | `H3\|_GG`                   | H3 UNIT | 종성 ㄲ (U+11A9) |
| 0x22 | `"`   | `0xB7`                      | literal | `·` MIDDLE DOT (U+00B7) |
| 0x23 | `#`   | `T ? H3\|_J : 0x23`         | cond  | composing→종성 ㅈ (U+11BD); else `#` |
| 0x24 | `$`   | `T ? H3\|0x1F4 : 0x24`      | cond  | composing→ unit id 0x1F4 = **종성 ㅎ** (`_H`, U+11C2) **(?)** ; else `$` |
| 0x25 | `%`   | `C0\|0xA`                   | ctrl  | command 0x0A (editor) |
| 0x26 | `&`   | `0x2015`                    | literal | `―` HORIZONTAL BAR (U+2015) |
| 0x27 | `'`   | `H3\|T_`                    | H3 UNIT | 초성 ㅌ (U+1110) |
| 0x28 | `(`   | `0x27`                      | literal | `'` APOSTROPHE (U+0027) |
| 0x29 | `)`   | `0x7E`                      | literal | `~` TILDE (U+007E) |
| 0x2A | `*`   | `0x2026`                    | literal | `…` HORIZONTAL ELLIPSIS (U+2026) |
| 0x2B | `+`   | `0x2B`                      | literal | `+` (U+002B) |
| 0x2C | `,`   | `0x2C`                      | literal | `,` (U+002C) |
| 0x2D | `-`   | `0x29`                      | literal | `)` RIGHT PAREN (U+0029) |
| 0x2E | `.`   | `0x2E`                      | literal | `.` (U+002E) |
| 0x2F | `/`   | `H3\|O_`                    | H3 UNIT | 중성 ㅗ (U+1169) |
| 0x30 | `0`   | `H3\|K_`                    | H3 UNIT | 초성 ㅋ (U+110F) |
| 0x31 | `1`   | `H3\|_H`                    | H3 UNIT | 종성 ㅎ (U+11C2) |
| 0x32 | `2`   | `H3\|_SS`                   | H3 UNIT | 종성 ㅆ (U+11BB) |
| 0x33 | `3`   | `H3\|_B`                    | H3 UNIT | 종성 ㅂ (U+11B8) |
| 0x34 | `4`   | `H3\|YO`                    | H3 UNIT | 중성 ㅛ (U+116D) |
| 0x35 | `5`   | `H3\|YU`                    | H3 UNIT | 중성 ㅠ (U+1172) |
| 0x36 | `6`   | `H3\|YA`                    | H3 UNIT | 중성 ㅑ (U+1163) |
| 0x37 | `7`   | `H3\|YE`                    | H3 UNIT | 중성 ㅖ (U+1168) |
| 0x38 | `8`   | `H3\|EUI`                   | H3 UNIT | 중성 ㅢ (U+1174) |
| 0x39 | `9`   | `H3\|U_`                    | H3 UNIT | 중성 ㅜ (U+116E) |
| 0x3A | `:`   | `0x34`                      | literal | `4` (U+0034) |
| 0x3B | `;`   | `H3\|B_`                    | H3 UNIT | 초성 ㅂ (U+1107) |
| 0x3C | `<`   | `T ? C0\|0xF : 0x5B`        | cond  | composing→cmd 0x0F; else `[` (U+005B) |
| 0x3D | `=`   | `0x3E`                      | literal | `>` (U+003E) |
| 0x3E | `>`   | `T ? C0\|0xC : 0x5D`        | cond  | composing→cmd 0x0C; else `]` (U+005D) |
| 0x3F | `?`   | `0x21`                      | literal | `!` (U+0021) |
| 0x40 | `@`   | `T ? H3\|_RG : 0x40`        | cond  | composing→종성 ㄺ (U+11B0); else `@` |
| 0x41 | `A`   | `H3\|_D`                    | H3 UNIT | 종성 ㄷ (U+11AE) |
| 0x42 | `B`   | `0x3F`                      | literal | `?` (U+003F) |
| 0x43 | `C`   | `H3\|_K`                    | H3 UNIT | 종성 ㅋ (U+11BF) |
| 0x44 | `D`   | `H3\|_RB`                   | H3 UNIT | 종성 ㄼ (U+11B2) |
| 0x45 | `E`   | `H3\|_NJ`                   | H3 UNIT | 종성 ㄵ (U+11AC) |
| 0x46 | `F`   | `H3\|_RM`                   | H3 UNIT | 종성 ㄻ (U+11B1) |
| 0x47 | `G`   | `H3\|YAE`                   | H3 UNIT | 중성 ㅒ (U+1164) |
| 0x48 | `H`   | `0x30`                      | literal | `0` (U+0030) |
| 0x49 | `I`   | `0x37`                      | literal | `7` (U+0037) |
| 0x4A | `J`   | `0x31`                      | literal | `1` (U+0031) |
| 0x4B | `K`   | `0x32`                      | literal | `2` (U+0032) |
| 0x4C | `L`   | `0x33`                      | literal | `3` (U+0033) |
| 0x4D | `M`   | `0x22`                      | literal | `"` (U+0022) |
| 0x4E | `N`   | `0x2D`                      | literal | `-` (U+002D) |
| 0x4F | `O`   | `0x38`                      | literal | `8` (U+0038) |
| 0x50 | `P`   | `0x39`                      | literal | `9` (U+0039) |
| 0x51 | `Q`   | `H3\|_P`                    | H3 UNIT | 종성 ㅍ (U+11C1) |
| 0x52 | `R`   | `H3\|_RH`                   | H3 UNIT | 종성 ㅀ (U+11B6) |
| 0x53 | `S`   | `H3\|_NH`                   | H3 UNIT | 종성 ㄶ (U+11AD) |
| 0x54 | `T`   | `C0\|2`                     | ctrl  | command 2 (editor) |
| 0x55 | `U`   | `0x36`                      | literal | `6` (U+0036) |
| 0x56 | `V`   | `H3\|_GS`                   | H3 UNIT | 종성 ㄳ (U+11AA) |
| 0x57 | `W`   | `H3\|_T`                    | H3 UNIT | 종성 ㅌ (U+11C0) |
| 0x58 | `X`   | `H3\|_BS`                   | H3 UNIT | 종성 ㅄ (U+11B9) |
| 0x59 | `Y`   | `0x35`                      | literal | `5` (U+0035) |
| 0x5A | `Z`   | `H3\|_C`                    | H3 UNIT | 종성 ㅊ (U+11BE) |
| 0x5B | `[`   | `0x28`                      | literal | `(` (U+0028) |
| 0x5C | `\`   | `0x3A`                      | literal | `:` (U+003A) |
| 0x5D | `]`   | `0x3C`                      | literal | `<` (U+003C) |
| 0x5E | `^`   | `0x3D`                      | literal | `=` (U+003D) |
| 0x5F | `_`   | `0x3B`                      | literal | `;` (U+003B) |
| 0x60 | `` ` `` | `T ? C0\|0xE : 0x2A`      | cond  | composing→cmd 0x0E; else `*` (U+002A) |
| 0x61 | `a`   | `H3\|_Q`                    | H3 UNIT | 종성 ㅇ (U+11BC) |
| 0x62 | `b`   | `H3\|0x810000`              | H3 UNIT | **filler/special unit** id 0x810000 — 중성 채움/filler **(?)** (see §A.3) |
| 0x63 | `c`   | `H3\|E_`                    | H3 UNIT | 중성 ㅔ (U+1166) |
| 0x64 | `d`   | `H3\|I_`                    | H3 UNIT | 중성 ㅣ (U+1175) |
| 0x65 | `e`   | `H3\|YEO`                   | H3 UNIT | 중성 ㅕ (U+1167) |
| 0x66 | `f`   | `H3\|A_`                    | H3 UNIT | 중성 ㅏ (U+1161) |
| 0x67 | `g`   | `H3\|0x820000`              | H3 UNIT | **filler/special unit** id 0x820000 **(?)** (see §A.3) |
| 0x68 | `h`   | `H3\|N_`                    | H3 UNIT | 초성 ㄴ (U+1102) |
| 0x69 | `i`   | `H3\|M_`                    | H3 UNIT | 초성 ㅁ (U+1106) |
| 0x6A | `j`   | `H3\|Q_`                    | H3 UNIT | 초성 ㅇ (U+110B) |
| 0x6B | `k`   | `H3\|G_`                    | H3 UNIT | 초성 ㄱ (U+1100) |
| 0x6C | `l`   | `H3\|J_`                    | H3 UNIT | 초성 ㅈ (U+110C) |
| 0x6D | `m`   | `H3\|H_`                    | H3 UNIT | 초성 ㅎ (U+1112) |
| 0x6E | `n`   | `H3\|S_`                    | H3 UNIT | 초성 ㅅ (U+1109) |
| 0x6F | `o`   | `H3\|C_`                    | H3 UNIT | 초성 ㅊ (U+110E) |
| 0x70 | `p`   | `H3\|P_`                    | H3 UNIT | 초성 ㅍ (U+1111) |
| 0x71 | `q`   | `H3\|_S`                    | H3 UNIT | 종성 ㅅ (U+11BA) |
| 0x72 | `r`   | `H3\|AE`                    | H3 UNIT | 중성 ㅐ (U+1162) |
| 0x73 | `s`   | `H3\|_N`                    | H3 UNIT | 종성 ㄴ (U+11AB) |
| 0x74 | `t`   | `H3\|EO`                    | H3 UNIT | 중성 ㅓ (U+1165) |
| 0x75 | `u`   | `H3\|D_`                    | H3 UNIT | 초성 ㄷ (U+1103) |
| 0x76 | `v`   | `H3\|0x800000`              | H3 UNIT | **filler/special unit** id 0x800000 — 초성 채움/filler **(?)** (see §A.3) |
| 0x77 | `w`   | `H3\|_R`                    | H3 UNIT | 종성 ㄹ (U+11AF) |
| 0x78 | `x`   | `H3\|_G`                    | H3 UNIT | 종성 ㄱ (U+11A8) |
| 0x79 | `y`   | `H3\|R_`                    | H3 UNIT | 초성 ㄹ (U+1105) |
| 0x7A | `z`   | `H3\|_M`                    | H3 UNIT | 종성 ㅁ (U+11B7) |
| 0x7B | `{`   | `0x25`                      | literal | `%` (U+0025) |
| 0x7C | `\|`  | `0x5C`                      | literal | `\` (U+005C) |
| 0x7D | `}`   | `0x2F`                      | literal | `/` (U+002F) |
| 0x7E | `~`   | `0x203B`                    | literal | `※` REFERENCE MARK (U+203B) |

Total `<Key>` rows: **94** (0x21..0x7E inclusive). Of these:
- **Hangul H3 units: 49 keys** (47 with named mnemonic + the 2 `T ?` jamo-shifted `_J`/`0x1F4`
  count once each as conditional but emit jamo; plus 3 filler/special `0x800000/810000/820000`).
  Counting strictly `H3|` keys (unconditional): **46**; conditional jamo (`#`,`$`,`@`): **3** more.
- **Conditional `T ? :` keys: 6** (`#`, `$`, `<`, `>`, `@`, `` ` ``).
- **Control `C0|` keys: 3** (`%`=C0|0xA, `T`=C0|2) + the C0 branches inside conditionals.
- **Literal symbol/digit keys: the remainder.**

### A.3 The three raw-id units (`0x800000`, `0x810000`, `0x820000`) and `0x1F4`

These are **not** standard mnemonics, so they are referenced by raw internal unit id. Based on
날개셋's internal layout (high-bit-flagged synthetic units) and their physical key positions, my
best reading:

- key `v` `H3|0x800000` — a **초성-class filler/special** (likely **ㅇ-less 초성 채움** or an
  *old-hangul* leading slot). Sits in the left-hand 종성 zone physically but is flagged as a
  special unit. **(?)**
- key `b` `H3|0x810000` — a **중성-class filler** (중성 채움, jungseong filler U+1160 territory)
  or special medial. **(?)**
- key `g` `H3|0x820000` — a **종성-class filler / special** trailing slot. **(?)**
- `0x1F4` (= decimal 500) on key `$`: decimal **500** is the **갈마들이 sentinel** `b="500"`
  used throughout `UnitMixTable` (see §B). So `T ? H3|0x1F4 : 0x24` means *"while composing,
  this key feeds the **500 toggle token**"* — i.e. it is the **갈마들이 double/tense toggle key**,
  not literally 종성 ㅎ. **Correction to the table row 0x24: `0x1F4` = unit 500 = 갈마들이
  toggle token, NOT 종성 ㅎ.** This makes `$` the "repeat/toggle" key that turns
  ㄱ→ㄲ, ㄲ→ㄱ, ㅅ→ㅆ, ㄹ+→ㄺ etc. via the `b=500` UnitMix rows. **(high confidence given §B)**

> NOTE: the `0x1F4`/500 reading is well-supported by `UnitMixTable` (every `b="500"` row).
> The `0x800000/810000/820000` filler readings are the least certain part of this doc.

### A.4 3-set keyboard layout (which hand)

Per 세벌식 convention and confirmed by the value classes:

**오른손 / RIGHT HAND = 초성 (leading consonants)** — home & right block, lowercase letter keys:
`k=ㄱ h=ㄴ u=ㄷ y=ㄹ i=ㅁ ;=ㅂ n=ㅅ j=ㅇ l=ㅈ o=ㅊ 0=ㅋ '=ㅌ p=ㅍ m=ㅎ` (14 초성).

**가운데 / MIDDLE = 중성 (vowels)** — right-of-center vowel block:
`f=ㅏ r=ㅐ 6=ㅑ G=ㅒ t=ㅓ c=ㅔ e=ㅕ 7=ㅖ /=ㅗ 4=ㅛ 9=ㅜ 5=ㅠ 8=ㅢ d=ㅣ` (+ combining via UnitMix:
ㅘㅙㅚㅝㅞㅟ). 17 중성 total counting compounds.

**왼손 / LEFT HAND = 종성 (trailing consonants)** — left block, number row & left letters:
`x=ㄱ !=ㄲ V=ㄳ s=ㄴ E=ㄵ S=ㄶ A=ㄷ w=ㄹ @=ㄺ F=ㄻ D=ㄼ R=ㅀ z=ㅁ 3=ㅂ X=ㅄ q=ㅅ 2=ㅆ a=ㅇ
#=ㅈ Z=ㅊ C=ㅋ W=ㅌ Q=ㅍ 1=ㅎ` (+ RS/RT/RP via UnitMix). 27 종성 total.

This matches "초성 오른쪽, 중성 가운데, 종성 왼쪽" exactly — it is a 공병우-식 3-set
("세벌식") layout, here a **맞춤(custom)** variant of 세벌식 최종.

---

## B. UNIT INVENTORY

Conjoining-jamo codepoints below are cross-validated against this config's own
`FinalConvTable` (conjoining→compatibility) where a row exists. "Modern idx" = 0-based index
within the modern choseong(19)/jungseong(21)/jongseong(28, 0=none) tables used by the
Hangul-syllable NFC algorithm.

### B.1 초성 (CHO) units

| Mnemonic | Jamo | Conjoining | Compat (U+31xx) | Modern CHO idx |
|----------|------|-----------|-----------------|----------------|
| `G_` | ㄱ | U+1100 | U+3131 | 0 |
| `GG` | ㄲ | U+1101 | U+3132 | 1 |
| `N_` | ㄴ | U+1102 | U+3134 | 2 |
| `D_` | ㄷ | U+1103 | U+3137 | 3 |
| `DD` | ㄸ | U+1104 | U+3138 | 4 |
| `R_` | ㄹ | U+1105 | U+3139 | 5 |
| `M_` | ㅁ | U+1106 | U+3141 | 6 |
| `B_` | ㅂ | U+1107 | U+3142 | 7 |
| `BB` | ㅃ | U+1108 | U+3143 | 8 |
| `S_` | ㅅ | U+1109 | U+3145 | 9 |
| `SS` | ㅆ | U+110A | U+3146 | 10 |
| `Q_` | ㅇ | U+110B | U+3147 | 11 |
| `J_` | ㅈ | U+110C | U+3148 | 12 |
| `JJ` | ㅉ | U+110D | U+3149 | 13 |
| `C_` | ㅊ | U+110E | U+314A | 14 |
| `K_` | ㅋ | U+110F | U+314B | 15 |
| `T_` | ㅌ | U+1110 | U+314C | 16 |
| `P_` | ㅍ | U+1111 | U+314D | 17 |
| `H_` | ㅎ | U+1112 | U+314E | 18 |

### B.2 중성 (JUNG) units

| Mnemonic | Jamo | Conjoining | Compat (U+31xx) | Modern JUNG idx |
|----------|------|-----------|-----------------|-----------------|
| `A_`  | ㅏ | U+1161 | U+314F | 0 |
| `AE`  | ㅐ | U+1162 | U+3150 | 1 |
| `YA`  | ㅑ | U+1163 | U+3151 | 2 |
| `YAE` | ㅒ | U+1164 | U+3152 | 3 |
| `EO`  | ㅓ | U+1165 | U+3153 | 4 |
| `E_`  | ㅔ | U+1166 | U+3154 | 5 |
| `YEO` | ㅕ | U+1167 | U+3155 | 6 |
| `YE`  | ㅖ | U+1168 | U+3156 | 7 |
| `O_`  | ㅗ | U+1169 | U+3157 | 8 |
| `WA`  | ㅘ | U+116A | U+3158 | 9  (O_+A_) |
| `WAE` | ㅙ | U+116B | U+3159 | 10 (O_+AE) |
| `OI`  | ㅚ | U+116C | U+315A | 11 (O_+I_) |
| `YO`  | ㅛ | U+116D | U+315B | 12 |
| `U_`  | ㅜ | U+116E | U+315C | 13 |
| `UEO` | ㅝ | U+116F | U+315D | 14 (U_+EO) |
| `WE`  | ㅞ | U+1170 | U+315E | 15 (U_+E_) |
| `WI`  | ㅟ | U+1171 | U+315F | 16 (U_+I_) |
| `YU`  | ㅠ | U+1172 | U+3160 | 17 |
| `EU`  | ㅡ | U+1173 | U+3161 | 18 (VirtualUnit only) |
| `EUI` | ㅢ | U+1174 | U+3162 | 19 |
| `I_`  | ㅣ | U+1175 | U+3163 | 20 |

(`EU` ㅡ has **no direct key**; it exists only as `VirtualUnit from=130`, see §B.4. There is
likewise no plain ㅡ key in the KeyTable — ㅡ is produced via the virtual-unit/모아치기 path.)
**(?) the absence of a standalone ㅡ key is notable — flag.**

### B.3 종성 (JONG) units

| Mnemonic | Jamo | Conjoining | Compat (U+31xx) | Modern JONG idx |
|----------|------|-----------|-----------------|-----------------|
| `_G`  | ㄱ | U+11A8 | U+3131 | 1 |
| `_GG` | ㄲ | U+11A9 | U+3132 | 2 |
| `_GS` | ㄳ | U+11AA | U+3133 | 3 |
| `_N`  | ㄴ | U+11AB | U+3134 | 4 |
| `_NJ` | ㄵ | U+11AC | U+3135 | 5 |
| `_NH` | ㄶ | U+11AD | U+3136 | 6 |
| `_D`  | ㄷ | U+11AE | U+3137 | 7 |
| `_R`  | ㄹ | U+11AF | U+3139 | 8 |
| `_RG` | ㄺ | U+11B0 | U+313A | 9  (R_+G via @) |
| `_RM` | ㄻ | U+11B1 | U+313B | 10 |
| `_RB` | ㄼ | U+11B2 | U+313C | 11 |
| `_RS` | ㄽ | U+11B3 | U+313D | 12 (RS = R_+S_ UnitMix) |
| `_RT` | ㄾ | U+11B4 | U+313E | 13 (RT = R_+T_ UnitMix) |
| `_RP` | ㄿ | U+11B5 | U+313F | 14 (RP = R_+P_ UnitMix) |
| `_RH` | ㅀ | U+11B6 | U+3140 | 15 |
| `_M`  | ㅁ | U+11B7 | U+3141 | 16 |
| `_B`  | ㅂ | U+11B8 | U+3142 | 17 |
| `_BS` | ㅄ | U+11B9 | U+3144 | 18 |
| `_S`  | ㅅ | U+11BA | U+3145 | 19 |
| `_SS` | ㅆ | U+11BB | U+3146 | 20 |
| `_Q`  | ㅇ | U+11BC | U+3147 | 21 |
| `_J`  | ㅈ | U+11BD | U+3148 | 22 |
| `_C`  | ㅊ | U+11BE | U+314A | 23 |
| `_K`  | ㅋ | U+11BF | U+314B | 24 |
| `_T`  | ㅌ | U+11C0 | U+314C | 25 |
| `_P`  | ㅍ | U+11C1 | U+314D | 26 |
| `_H`  | ㅎ | U+11C2 | U+314E | 27 |

### B.4 VirtualUnitTable (모아치기 stroke→unit promotion)

| from | to  | unit class | meaning |
|------|-----|-----------|---------|
| 128  | `O_` | JUNG | virtual stroke 128 → ㅗ (used to seed ㅘ/ㅙ/ㅚ in 모아치기) |
| 129  | `U_` | JUNG | virtual stroke 129 → ㅜ (seeds ㅝ/ㅞ/ㅟ) |
| 130  | `EU` | JUNG | virtual stroke 130 → ㅡ (the only way to get ㅡ; seeds ㅢ) |

These let a half-vowel keystroke be promoted to the right JUNG when combined.

### B.5 UnitMixTable (combination rules)

Two kinds of rows: **a+b (literal two-jamo merge)** and **a + 500 (갈마들이 toggle)**.
`b="500"` = the 갈마들이 sentinel (the `0x1F4` token, key `$`). 갈마들이 pairs marked **(갈마)**.

CHO doublings:
| a | b | → | note |
|---|---|---|------|
| G_ | G_ | GG | ㄱ+ㄱ→ㄲ (repeat key) |
| G_ | 500 | GG | **(갈마)** ㄱ +toggle→ ㄲ |
| GG | 500 | G_ | **(갈마)** ㄲ +toggle→ ㄱ (back) |
| D_ | D_ | DD | ㄷ+ㄷ→ㄸ |
| D_ | 500 | DD | **(갈마)** |
| DD | 500 | D_ | **(갈마)** |
| B_ | B_ | BB | ㅂ+ㅂ→ㅃ |
| B_ | 500 | BB | **(갈마)** |
| BB | 500 | B_ | **(갈마)** |
| S_ | S_ | SS | ㅅ+ㅅ→ㅆ |
| S_ | 500 | SS | **(갈마)** |
| SS | 500 | S_ | **(갈마)** |
| J_ | J_ | JJ | ㅈ+ㅈ→ㅉ |
| J_ | 500 | JJ | **(갈마)** |
| JJ | 500 | J_ | **(갈마)** |

JUNG compounds:
| a | b | → | jamo |
|---|---|---|------|
| O_ | A_ | WA  | ㅗ+ㅏ→ㅘ |
| O_ | AE | WAE | ㅗ+ㅐ→ㅙ |
| O_ | I_ | OI  | ㅗ+ㅣ→ㅚ |
| U_ | EO | UEO | ㅜ+ㅓ→ㅝ |
| U_ | E_ | WE  | ㅜ+ㅔ→ㅞ |
| U_ | I_ | WI  | ㅜ+ㅣ→ㅟ |

JONG compounds (ㄹ-clusters that aren't on dedicated keys):
| a | b | → | jamo |
|---|---|---|------|
| R_ | S_ | RS | ㄹ+ㅅ→ㄽ |
| R_ | T_ | RT | ㄹ+ㅌ→ㄾ |
| R_ | P_ | RP | ㄹ+ㅍ→ㄿ |

> NOTE: `R_`/`S_`/`T_`/`P_` in the JONG mix rows refer to the trailing-position
> equivalents (날개셋 resolves the position by automaton context); RS/RT/RP are the
> 종성 results ㄽ/ㄾ/ㄿ above. The other ㄹ-clusters ㄺ/ㄻ/ㄼ/ㅀ are on dedicated keys
> (`@ F D R`).

### B.6 갈마들이 (b=500) summary

The five tense pairs are toggled by the **500 token (key `$`)**:
ㄱ↔ㄲ, ㄷ↔ㄸ, ㅂ↔ㅃ, ㅅ↔ㅆ, ㅈ↔ㅉ. Pressing the base 초성 then `$` gives the tense; pressing
`$` again on a tense gives back the plain. (Same physical-key 반복 also works:
`G_+G_→GG` etc.) This is the "같은 자리 반복으로 된소리" 갈마들이 design.

---

## C. AUTOMATON BEHAVIOR

### C.1 The three states (`AutomataTable default="0"`)

```
state 0  "초기 상태"      value="1"  default="0"
state 1  "미완성 상태"    value="D==176&&A==176 || D==185&&A==185 ? 0
                                : A||B||C ? (A||D)&&(B||E) ? 2 : 1
                                : -2"
                          default="-1"
state 2  "한글 완성 상태"  value="A&&A!=500 ? 0 : B||C||A==500 ? 2 : -2"
                          default="0"
```

날개셋 automaton variable convention (per-keystroke evaluation):
- `A` = the **초성 (CHO)** component of the incoming/just-typed unit (nonzero if this key is a 초성),
- `B` = the **중성 (JUNG)** component,
- `C` = the **종성 (JONG)** component,
- `D`,`E` = the **already-present** CHO / JUNG of the syllable under composition (the "stored" state),
- a return value of a **state number** = transition to that state; **`-2`** = *commit current
  syllable and restart with this key*; **`-1`** = error/flush; **`0/1/2`** literal target states.
- `500` = the 갈마들이 sentinel token; `176` and `185` are specific unit ids
  (176 ≈ ㅇ-class, 185 ≈ a specific jamo) used to special-case "same filler twice" **(?)**.

Plain-language reading:

- **State 0 (초기/idle):** no syllable in progress. Any first jamo → go to state 1
  (`value="1"`). Begin a new syllable buffer.
- **State 1 (미완성/incomplete):** a syllable is being built but is not yet a "complete"
  CV(C). On each new unit:
  - The guard `D==176&&A==176 || D==185&&A==185 ? 0` handles a *filler-repeat / special reset*
    edge case → drop to state 0. **(?)**
  - Else if the key carries any jamo (`A||B||C`): if we now have **both** a CHO **and** a JUNG
    (`(A||D)&&(B||E)`) → the syllable is at least CV → go to **state 2 (완성)**; otherwise stay
    in **state 1**.
  - Else (`-2`) → **commit and restart** (the key was not a jamo, e.g. a symbol/space).
- **State 2 (한글 완성/complete):** a full CV or CVC syllable exists and is displayable.
  - `A&&A!=500 ? 0`: if a **new real 초성** arrives (and it's not the 500 toggle) → **commit the
    current syllable, start a new one at state 0** (this is the auto-boundary: 초성 after a
    complete syllable begins the next syllable — the hallmark of a 3-set/이어치기 layout, since
    초성 and 종성 are distinct keys so the machine always knows whether a consonant is a 받침 or
    a new 초성).
  - `B||C||A==500 ? 2`: a new **중성**, a new **종성**, *or the 500 갈마들이 toggle* → stay in
    state 2 and **attach** (받침 attaches; 갈마들이 re-tenses the current consonant in place).
  - else `-2`: commit & restart.

### C.2 Intended typing model

- **이어치기 (sequential) is the natural model** for this 3-set layout: because 초성, 중성,
  종성 live on **physically different keys**, the automaton never has to guess role from order.
  A consonant key in the 종성 zone is always a 받침; a consonant in the 초성 zone always starts
  a (new) syllable. **모아치기 (chord/同時치기) is also supported** via the `VirtualUnitTable`
  (half-vowel promotion 128/129/130) and `flagex="1"` on the generator, letting near-simultaneous
  strokes coalesce into one syllable.
- **When a syllable commits:** (a) when a new **초성** arrives while in state 2
  (`A&&A!=500 ? 0`), the previous syllable is flushed and a fresh one starts; (b) when a
  **non-jamo** key (symbol, digit, space, the literal branch of a `T ? :` key) arrives,
  `-2` flushes the syllable then emits the literal; (c) on focus loss / explicit commit.
- **How 받침 attaches:** in state 2, a **종성** unit (`C`) stays in state 2 and is appended as
  the trailing consonant. A second compatible 종성 forms a cluster *only via* a dedicated
  cluster key or a `UnitMix` JONG rule (RS/RT/RP); otherwise the standard 받침-overflow rule
  applies — but note that because this is a 3-set layout, a following **초성** does **not**
  steal the 받침 (no 도깨비불/ghost-letter ambiguity), unlike 두벌식.
- **갈마들이 same-key toggle:** the 500 token (key `$`, or repeating the same consonant key)
  toggles plain↔tense 초성 (ㄱ↔ㄲ …) and, for ㄹ, lets you build single→double/compound 받침
  via the RS/RT/RP and the dedicated cluster keys. In state 2, `A==500` keeps you in state 2 and
  rewrites the current consonant in place rather than starting a new syllable.

### C.3 FinalConvTable role

`FinalConvTable` (167 rows) maps a **conjoining jamo** (U+1100/U+11xx, plus PUA old-hangul ids
0xA9xx/0xD7xx/0xEAxx/0xECxx) to its **compatibility jamo** (U+31xx). It is applied **at final
commit when a syllable consists of a lone jamo** (only a 초성, or only a 중성, or only a 종성
with no full syllable). Instead of emitting a "naked" conjoining jamo (which renders as an
isolated half-width combining form), the IME substitutes the standalone **compatibility jamo**.

Examples straight from the table:
- lone 초성 ㄱ U+1100 → **ㄱ U+3131**; ㄲ U+1101 → **ㄲ U+3132**; ㅎ U+1112 → **ㅎ U+314E**.
- lone 중성 ㅏ U+1161 → **ㅏ U+314F**; ㅢ U+1174 → **ㅢ U+3162**.
- lone 종성 ㄱ U+11A8 → **ㄱ U+3131**; ㄺ U+11B0 → **ㄺ U+313A**; ㅄ U+11B9 → **ㅄ U+3144**.
- old-hangul examples: ㅿ-trailing U+11EB → **ㅿ U+317F**; ㆁ (옛이응) U+11F0 → **U+3181**;
  ㅸ U+112B → **U+3178**; PUA U+EA40 → **ㄾ U+313E** (an old-hangul cluster mapped to compat).

So FinalConv is the "show a real standalone letter, not a combining stub" finalizer for jamo
that never completed into a syllable.

---

## D. TEST VECTORS

Conventions:
- "Keys" = the **physical US-QWERTY keys** the user presses (this layout maps QWERTY key→jamo
  per §A). Space = explicit commit/space.
- For each: keystroke string, the per-key jamo, the composing buffer, and the final committed
  string with U+ codepoints.
- These are **derivations**; uncertain ones are flagged **(?)**.

Key→jamo quick ref (from §A): `k`=초ㄱ `h`=초ㄴ `u`=초ㄷ `y`=초ㄹ `i`=초ㅁ `;`=초ㅂ `n`=초ㅅ
`j`=초ㅇ `l`=초ㅈ `o`=초ㅊ `0`=초ㅋ `'`=초ㅌ `p`=초ㅍ `m`=초ㅎ ‖ `f`=ㅏ `r`=ㅐ `6`=ㅑ `G`=ㅒ
`t`=ㅓ `c`=ㅔ `e`=ㅕ `7`=ㅖ `/`=ㅗ `4`=ㅛ `9`=ㅜ `5`=ㅠ `8`=ㅢ `d`=ㅣ ‖ `x`=종ㄱ `!`=종ㄲ
`V`=종ㄳ `s`=종ㄴ `E`=종ㄵ `S`=종ㄶ `A`=종ㄷ `w`=종ㄹ `@`=종ㄺ `F`=종ㄻ `D`=종ㄼ `R`=종ㅀ
`z`=종ㅁ `3`=종ㅂ `X`=종ㅄ `q`=종ㅅ `2`=종ㅆ `a`=종ㅇ `#`=종ㅈ `Z`=종ㅊ `C`=종ㅋ `W`=종ㅌ
`Q`=종ㅍ `1`=종ㅎ ‖ `$`=갈마들이 토글(500).

| # | Keys | Per-key jamo | Composing | Final string | Codepoints | Conf |
|---|------|--------------|-----------|--------------|-----------|------|
| 1  | `k f` | 초ㄱ, ㅏ | ㄱ→가 | **가** | U+AC00 | high |
| 2  | `h f` | 초ㄴ, ㅏ | ㄴ→나 | **나** | U+B098 | high |
| 3  | `u f` | 초ㄷ, ㅏ | ㄷ→다 | **다** | U+B2E4 | high |
| 4  | `k f x` | 초ㄱ, ㅏ, 종ㄱ | 가→각 | **각** | U+AC01 | high |
| 5  | `k f s` | 초ㄱ, ㅏ, 종ㄴ | 가→간 | **간** | U+AC04 | high |
| 6  | `i f s` | 초ㅁ, ㅏ, 종ㄴ | 마→만 | **만** | U+B9CC | high |
| 7  | `h f s` | 초ㄴ, ㅏ, 종ㄴ + commit | 나→난 | **난** | U+B09C | high |
| 8  | `k / f` | 초ㄱ, ㅗ, ㅏ(→ㅘ) | 고→과 | **과** | U+ACFC | high |
| 9  | `k 9 t` | 초ㄱ, ㅜ, ㅓ(→ㅝ) | 구→궈 | **궈** | U+AD88 | high |
| 10 | `j 9 t` | 초ㅇ, ㅜ, ㅓ(→ㅝ) | 우→워 | **워** | U+C6CC | high |
| 11 | `j / d` | 초ㅇ, ㅗ, ㅣ(→ㅚ) | 오→외 | **외** | U+C678 | high |
| 12 | `j 9 d` | 초ㅇ, ㅜ, ㅣ(→ㅟ) | 우→위 | **위** | U+C704 | high |
| 13 | `j 8` | 초ㅇ, ㅢ | 의 | **의** | U+C758 | high |
| 14 | `k f @` | 초ㄱ, ㅏ, 종ㄺ | 가→갉 | **갉** | U+AC09 | high |
| 15 | `u f F` | 초ㄷ, ㅏ, 종ㄻ | 다→닮 | **닮** | U+B2EE | high |
| 16 | `k f V` | 초ㄱ, ㅏ, 종ㄳ | 가→갃 | **갃** | U+AC03 | med |
| 17 | `j f s S?` | 초ㅇ, ㅏ, 종ㄶ (`S`) | 아→앉?… | **않** (`j f S`) | U+C54A | high |
| 18 | `k k f` | 초ㄱ,초ㄱ(→ㄲ),ㅏ | (ㄲ)→까 | **까** | U+AE4C | high |
| 19 | `k $ f` | 초ㄱ, 갈마(500)→ㄲ, ㅏ | 까 | **까** | U+AE4C | high |
| 20 | `n $ f` | 초ㅅ, 갈마→ㅆ, ㅏ | 싸 | **싸** | U+C2F8 | high |
| 21 | `k $ $ f` | ㄱ→ㄲ→(toggle back)ㄱ, ㅏ | 가 | **가** | U+AC00 | med |
| 22 | `u $ f` | 초ㄷ→ㄸ, ㅏ | 따 | **따** | U+B530 | high |
| 23 | `l $ f` | 초ㅈ→ㅉ, ㅏ | 짜 | **짜** | U+C9DC | high |
| 24 | `k f k f` | 가 / 가 (2nd 초성 commits 1st) | 가│가 | **가가** | U+AC00 U+AC00 | high |
| 25 | `k f h f` | 가, then 초ㄴ→commit, 나 | 가│나 | **가나** | U+AC00 U+B098 | high |
| 26 | `k` then space | 초ㄱ alone → FinalConv | ㄱ | **ㄱ** (compat) | U+3131 (+ space) | high |
| 27 | `f` alone | ㅏ alone → FinalConv | ㅏ | **ㅏ** | U+314F | high |
| 28 | `x` alone | 종ㄱ alone → FinalConv U+11A8→U+3131 | ㄱ | **ㄱ** | U+3131 | med |
| 29 | `@` alone | 종ㄺ alone → FinalConv U+11B0→U+313A | ㄺ | **ㄺ** | U+313A | med |
| 30 | `X` alone | 종ㅄ alone → FinalConv U+11B9→U+3144 | ㅄ | **ㅄ** | U+3144 | med |
| 31 | `k f x f` | 각 then ㅏ: ㄱ re-syllabifies?  3-set → 받침 stays, ㅏ has no 초성 | 각│ㅏ | **각ㅏ** (?) | U+AC01 U+314F | low |
| 32 | `y f w` | 초ㄹ, ㅏ, 종ㄹ | 라→랄 | **랄** | U+B784 | high |
| 33 | `y f w q` | 초ㄹ,ㅏ,종ㄹ,종ㅅ(→ㄽ via RS) | 랄→랈 | **랈** | U+B788 | med |
| 34 | `y f w W` | 초ㄹ,ㅏ,종ㄹ,종ㅌ(→ㄾ via RT) | 랄→랉 | **랉** | U+B789 | med |
| 35 | `i f Q` | 초ㅁ,ㅏ,종ㅍ | 마→맢 | **맢** | U+B9E2 | med |
| 36 | `v f` | 0x800000 filler 초, ㅏ → old-hangul lead **(?)** | (채움)+ㅏ | **ㅏ-syllable w/ filler choseong** (?) | U+115F U+1161 (?) | low |
| 37 | `g` after vowel | 0x820000 special 종 **(?)** | ? | **?** | ? | low |
| 38 | `b` mid-compose | 0x810000 중성 filler **(?)** | ? | **?** | ? | low |

### D.1 Old-hangul cases (best-effort, ≥3)

The FinalConvTable proves old-hangul *output* support, but this 맞춤 KeyTable has **no direct
keys** for ㅿ/ㆆ/ㅸ/옛이응 etc., so old-hangul syllables would require the filler units
(`v`/`b`/`g` = 0x800000/810000/820000) and/or 모아치기 of clusters not all of which are reachable
from the 47 letter keys. Three derivable cases:

| # | Keys | Intent | Final (best guess) | Codepoints | Conf |
|---|------|--------|--------------------|-----------|------|
| O1 | `v` then `f` | filler 초성 + ㅏ → "ㅇ-less" leading syllable (옛한글 채움) | bare ㅏ with choseong filler | U+115F U+1161 (or just **ㅏ** U+314F if collapsed) | low |
| O2 | `k f` + `b` + ... | inject 중성 filler 0x810000 between jamo | malformed/old medial | U+1160-based | low |
| O3 | lone `g` → FinalConv | a 종성 special id (0x820000) committed alone → compat jamo via table | one of U+31xx old-hangul compat (e.g. **ㅿ** U+317F / **ㆁ** U+3181) | low |

These three are **speculative** — the raw filler unit ids (0x800000/810000/820000) are the part
of the config I could not fully decode, so the exact old-hangul output is unverified.

### D.2 Notes / caveats on the vectors

- Vectors 1–25, 32 are high confidence (standard modern-syllable NFC composition).
- Vector 17: `S` = 종ㄶ directly (the cluster is a single key), so `j f S` → 않 U+C54A; do not
  type ㄴ+ㅎ separately.
- Vector 21 (double 갈마 toggle back to plain) assumes the `GG b=500 → G_` row fires on a second
  `$`; if the engine instead requires the same physical letter key, behavior differs — flag.
- Vector 24/25 rely on the **state-2 `A&&A!=500 ? 0` auto-commit**: a new 초성 finishes the prior
  syllable (no space needed). This is the central 3-set behavior to test.
- Vector 31 is genuinely uncertain: after a complete CVC (각), a bare **vowel** key (ㅏ, no
  preceding 초성) — a pure 3-set engine would *not* move the 받침; ㅏ would start a filler/낱자
  syllable → likely "각" + standalone "ㅏ". Some engines instead do 도깨비불. **Flag for runtime
  verification.**
- Codepoints for precomposed syllables computed via
  `0xAC00 + ((cho*21)+jung)*28 + jong` with the modern indices in §B.

---

## E. Open questions / could-not-fully-decode

1. **`0x800000` (`v`), `0x810000` (`b`), `0x820000` (`g`)** — raw synthetic unit ids. Read as
   초성/중성/종성 **채움(filler)** specials but exact emitted codepoints unverified. **Primary
   unknown.**
2. **`0x1F4` on key `$`** — decoded as the **500 갈마들이 toggle token** (consistent with all
   `b="500"` UnitMix rows). High confidence, but worth runtime confirmation vs. literal 종성 ㅎ.
3. **Automaton magic numbers `176`, `185`** in state 1 — special-cased unit ids (filler /
   ㅇ-repeat edge cases). Their precise jamo identity is a guess. **(?)**
4. **`C0|` command codes** (`C0|0xA`, `C0|2`, `C0|0xF`, `C0|0xC`, `C0|0xE`) and the shortcut
   `C0|0x82` (VK_HANJA) — editor/command actions, not text; their exact effects (newline, undo,
   redo, bracket-jump, 한자 conversion) are inferred, not decoded.
5. **No standalone ㅡ key** and **EU only as VirtualUnit 130** — confirms ㅡ is produced via the
   virtual/모아치기 path; the exact stroke to type a lone ㅡ from this KeyTable is unclear (no `at`
   maps to `H3|EU`). **(?)**
6. **`UserKeyTable doobeolchk="T<=1"`** and **`Bksp`** rules (`BkspAttach`, `ByUnitStep`,
   `BySyllable`, `ReverseJLTRN`) define backspace granularity (by-jamo vs by-syllable) — noted
   but not turned into test vectors.
