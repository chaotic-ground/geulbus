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
# 참고: 심볼/rules 파일은 setup-hanja-key.sh 와 공유한다(두 스크립트 모두 같은
# superset 을 쓴다). 옵션(gsettings)만 각자 관리하므로 서로 덮어써도 안전하다.
#
# 되돌리기:  scripts/setup-capslock-hangul.sh --revert
set -euo pipefail

# shellcheck source=scripts/lib-xkb-geulbus.sh
source "$(dirname "$0")/lib-xkb-geulbus.sh"

geulbus_xkb_toggle_option "geulbus:caps_hangul" "${1:-}"
