#!/usr/bin/env bash
# 오른쪽 Alt 를 한자 키로 (GNOME/Wayland, sudo 불필요).
# 한국 노트북에서 "한/영" 각인 키가 물리적으로는 오른쪽 Alt 인 경우가 많다.
#
#   - 오른쪽 Alt → Hangul_Hanja 키심 → geulbus 의 VK_HANJA(KEYCHAR) 배선에 걸려
#     조합 중 자음이면 특수문자, 음절이면 한자 후보창이 뜬다.
#   - 표준 XKB 옵션 korean:ralt_hanja 는 그룹 1에만 적용되므로, 엔진이 dvorak-mac
#     처럼 RALT=ISO_Level3_Shift 인 배열과 짝지어진 소스(그룹 2 이후)에서는 먹지
#     않는다. 여기서는 사용자 XKB rules 의 `:all` 지정자로 모든 그룹에 적용한다.
#
# 참고: 심볼/rules 파일은 setup-capslock-hangul.sh 와 공유한다(두 스크립트 모두
# 같은 superset 을 쓴다). 옵션(gsettings)만 각자 관리하므로 서로 덮어써도 안전하다.
#
# 되돌리기:  scripts/setup-hanja-key.sh --revert
set -euo pipefail

# shellcheck source=scripts/lib-xkb-geulbus.sh
source "$(dirname "$0")/lib-xkb-geulbus.sh"

geulbus_xkb_toggle_option "geulbus:ralt_hanja" "${1:-}"
