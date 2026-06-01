#!/usr/bin/env bash
# CapsLock 을 한/영 전환키로 (GNOME/Wayland). sudo 불필요.
#
#   - CapsLock 단독       → Hangul 키심 (잠금 동작 없음) → ibus 엔진이 한/영 토글
#   - Shift + CapsLock    → 평소 Caps Lock (대소문자 잠금)
#
# Wayland 에서는 CapsLock 잠금을 컴포지터(mutter)가 처리해 IME 가 가로챌 수 없으므로,
# 키보드(XKB) 레벨에서 단독 CapsLock 을 잠금이 아닌 Hangul 키심으로 바꾼다.
# 사용자 XKB 설정(~/.config/xkb)에 커스텀 옵션을 두고 gsettings 로 켠다.
#
# 되돌리기:  scripts/setup-capslock-hangul.sh --revert
set -euo pipefail

xkb_dir="${XDG_CONFIG_HOME:-$HOME/.config}/xkb"
key="org.gnome.desktop.input-sources"

if [[ "${1:-}" == "--revert" ]]; then
  # 옵션 목록에서 geulbus:caps_hangul 제거
  cur="$(gsettings get "$key" xkb-options)"
  new="$(printf '%s' "$cur" | sed "s/'geulbus:caps_hangul'//; s/, ,/,/g; s/\[, /[/; s/, \]/]/")"
  [[ "$new" == "@as []" || "$new" == "[]" ]] && gsettings reset "$key" xkb-options || gsettings set "$key" xkb-options "$new"
  rm -f "$xkb_dir/symbols/geulbus" "$xkb_dir/rules/evdev"
  echo "되돌림. 로그아웃/로그인 후 반영됩니다."
  exit 0
fi

mkdir -p "$xkb_dir/symbols" "$xkb_dir/rules"

cat > "$xkb_dir/symbols/geulbus" <<'XKB'
// CapsLock 단독 → 한/영(Hangul), Shift+CapsLock → 평소 Caps Lock(잠금).
partial modifier_keys
xkb_symbols "caps_hangul" {
    replace key <CAPS> {
        type[group1] = "TWO_LEVEL",
        symbols[group1] = [ Hangul, Caps_Lock ],
        actions[group1] = [ NoAction(), LockMods(modifiers = Lock) ]
    };
};
XKB

cat > "$xkb_dir/rules/evdev" <<'RULES'
! include %S/evdev

! option = symbols
  geulbus:caps_hangul = +geulbus(caps_hangul)
RULES

# 기존 옵션 보존하며 추가
cur="$(gsettings get "$key" xkb-options)"
if [[ "$cur" == "@as []" || "$cur" == "[]" ]]; then
  gsettings set "$key" xkb-options "['geulbus:caps_hangul']"
elif [[ "$cur" != *"geulbus:caps_hangul"* ]]; then
  gsettings set "$key" xkb-options "${cur%]}, 'geulbus:caps_hangul']"
fi

echo "적용값: $(gsettings get "$key" xkb-options)"
echo "반영하려면 로그아웃/로그인(또는 입력 소스 전환)을 한 번 하세요."
echo "되돌리려면: $0 --revert"
