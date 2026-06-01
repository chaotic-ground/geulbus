# 06 — 단축키/레이아웃 처리: 실기 진단과 설계 결정

geulbus 이 "범용 입력기"이면서, 사용자가 드보락 등 자기 레이아웃을 쓸 때 단축키도
그 레이아웃을 따르게 하려면 어떻게 해야 하는가. 실기 측정으로 확정했다.

## 실기 측정 (GNOME 49 / Wayland / IBus 1.5.33)

엔진의 `ProcessKeyEvent(keyval, keycode, state)` 가 실제로 받은 값
(`GEULBUS_DEBUG_KEYS=1` 로 stderr 로깅, Firefox 에서 실물 키보드로 입력).

`<layout>us</layout>` (또는 us 로 묶인 상태)에서:

| 누른 물리 키 | keyval | keycode(evdev) | 해석 |
|---|---|---|---|
| 드보락 P 자리 (= 물리 QWERTY-R) | `0x72 ('r')` | 19 | **QWERTY** keysym |
| Ctrl + 드보락 C 자리 (= 물리 QWERTY-I) | `0x69 ('i')` + Ctrl | 23 | **QWERTY** keysym |
| 물리 M | `0x6d ('m')` | 50 | QWERTY |
| 물리 T | `0x74 ('t')` | 20 | QWERTY |

**결론 1 (FACT):** 이 환경에선 geulbus(ibus 엔진) 활성 시 GNOME 이 XKB 를 **us(QWERTY)로
고정**한다. 사용자의 드보락은 무시되고 엔진은 QWERTY keysym 을 받는다. 이게 사용자가 본
"Alt+P 의도 → r 입력"의 정확한 원인. (gnome-shell `inputMethod.js` 는 현재 레이아웃의
keysym 을 넘기도록 돼 있으나, 입력 소스에 묶인 XKB 가 us 라 그 "현재 레이아웃"이 us 임.)

**결론 2 (FACT):** `keycode`(evdev)는 **레이아웃과 무관하게 물리 위치**로 정확히 온다
(드보락 P자리든 QWERTY 설정이든 물리 R = evdev 19). 날개셋이 Windows 에서 스캔코드로
물리 위치를 잡던 것과 동일. → **keycode 가 가장 신뢰할 수 있는 물리 위치 신호.**

## 왜 기존 접근(ForwardKeyEvent remap)이 실패했나 (FACT)

- gnome-shell `inputMethod.js` 의 `_onForwardKeyEvent` 는 forward 된 이벤트를 **keycode 로
  재구성**하고, 클라이언트가 자기 XKB 로 문자를 다시 만든다. IME 가 keyval 만 바꿔 보내도
  **keycode 가 이김** → 우리가 `p` 를 forward 해도 앱은 keycode(물리 R)+us 로 `r` 로 되돌림.
- 즉 **IME 의 ForwardKeyEvent 로 단축키 레이아웃을 바꾸는 건 Wayland 에서 원리적으로 불가.**
- English(Dvorak)이 단축키까지 잘 되는 이유: 그건 IME 가 아니라 `('xkb','us+dvorak')`
  순수 XKB 레이아웃이라, 컴포지터 레벨에서 드보락으로 처리되기 때문.

## `<layout>` 시도 (INCONCLUSIVE, 그러나 무의미)

`<layout>us(dvorak)</layout>` 로 바꿔 측정 시도 → 그 세션에서 활성 엔진이 hangul 로
바뀌어 키가 geulbus 에 도달하지 않아 측정 실패. 하지만 더 볼 필요 없음:
- `<layout>` 으로 특정 레이아웃을 강제하는 것 자체가 "드보락 강제 금지(범용)" 요구와 충돌.
- 따라서 geulbus 은 `<layout>default</layout>` (강제 안 함) 로 둔다.

## 설계 결정

**두 레이어를 분리한다(날개셋 모델을 Wayland 에 이식):**

1. **한글/영문 글자 = IME 가 CommitText 로 직접 출력.**
   날개셋처럼 **물리 위치 기준**으로 KeyTable 을 적용한다. 단, keyval 은 레이아웃에 따라
   흔들리므로(위 측정) **keycode(evdev) → 물리 QWERTY ASCII 표**로 변환해서 KeyTable 을
   조회한다. 그러면 사용자가 QWERTY 든 드보락이든 **세벌식 자리는 불변**(= 날개셋 동작).

2. **단축키(Ctrl/Alt/Super+키) = XKB(사용자 레이아웃) 가 담당.**
   geulbus 은 레이아웃을 강제하지 않고(`<layout>default`), 단축키는 **통과**(return false).
   사용자가 자기 단축키 레이아웃을 XKB 로 고른다(예: 시스템/입력소스를 드보락으로).
   IME 가 단축키를 흉내 내지 않는다 → ForwardKeyEvent remap 코드/ latin_entry 흉내 제거.

   ※ 단, 이 환경에서 "geulbus 활성 시 XKB 가 us 로 고정"되는 현상이 있으므로, 단축키가
   사용자 레이아웃을 따르게 하려면 사용자가 geulbus 입력 소스의 XKB 결합을 드보락으로
   맞추거나(레이아웃 변형 선택), 시스템 레벨에서 처리해야 함. 이는 geulbus 코드 밖의
   사용자 설정 영역. geulbus 은 "강제하지 않음 + 통과"로 올바르게 처신한다.

### evdev keycode → QWERTY ASCII 표 (측정·표준 evdev 기준)

행 기준(evdev set 1 아님, Linux evdev = XKB keycode - 8):
- 숫자열: 2..11 = `1234567890`, 12=`-`, 13=`=`
- 윗줄: 16..25 = `qwertyuiop`, 26=`[`, 27=`]`
- 가운뎃줄: 30..38 = `asdfghjkl`, 39=`;`, 40=`'`, 41=`` ` ``
- 아랫줄: 43=`\`, 44..53 = `zxcvbnm,./`
- (측정 교차검증: 19=r, 23=i, 50=m, 20=t, 36=j, 3=2, 5=4 모두 일치)

이 표는 **Shift 없는 baseline 물리 위치**만 준다. Shift 상태는 `state` 의 SHIFT 비트로
KeyTable 식의 P 변수에 전달(기존과 동일).

## 구현 작업 (다음 단계)
- core/ibus: KeyTable 조회 키를 `keyval` → `keycode→ascii(evdev 표)` 로 전환.
  (단, keycode 가 0 인 경우 = 프로그램적 주입/일부 클라이언트 → keyval 로 폴백.)
- ibus: simple-mode 의 `remap_shortcut` / `shortcut_layout` / `latin_entry` 단축키 변환 제거.
  단축키(ShortcutCombo)는 그냥 통과(return false).
- 컴포넌트: `<layout>default</layout>` 유지.
- 설정창: "영문 배치 항목"(단축키 변환용) 제거 또는 의미 재정의(한글 항목 선택만 남김).
