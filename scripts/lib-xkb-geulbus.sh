# geulbus 사용자 XKB 커스텀(심볼/rules) 공용 라이브러리. source 해서 쓴다.
#
# setup-capslock-hangul.sh 와 setup-hanja-key.sh 가 공유하는 파일을 항상
# 같은 superset 내용으로 쓰므로, 어느 스크립트를 나중에 실행해도 다른 쪽
# 정의가 사라지지 않는다. gsettings 옵션만 각자 켜고 끈다.

geulbus_xkb_write_files() {
    local xkb_dir="${XDG_CONFIG_HOME:-$HOME/.config}/xkb"
    mkdir -p "$xkb_dir/symbols" "$xkb_dir/rules"

    cat > "$xkb_dir/symbols/geulbus" <<'XKB'
// geulbus 사용자 XKB 심볼 (scripts/lib-xkb-geulbus.sh 가 생성).
//
// caps_hangul: CapsLock 단독 → 한/영(Hangul), Shift+CapsLock → 평소 Caps Lock(잠금).
// ralt_hanja:  오른쪽 Alt(흔히 "한/영" 각인) → 한자(Hangul_Hanja).
//              표준 korean:ralt_hanja 는 그룹 1에만 적용돼, dvorak-mac 처럼
//              RALT 가 ISO_Level3_Shift 인 배열(그룹 2 이후)에서는 먹지 않는다.
//              rules 의 `:all` 지정자로 모든 그룹에 이 섹션을 적용한다.

partial modifier_keys
xkb_symbols "caps_hangul" {
    replace key <CAPS> {
        type[group1] = "TWO_LEVEL",
        symbols[group1] = [ Hangul, Caps_Lock ],
        actions[group1] = [ NoAction(), LockMods(modifiers = Lock) ]
    };
};

partial modifier_keys
xkb_symbols "ralt_hanja" {
    replace key <RALT> {
        type[group1] = "ONE_LEVEL",
        symbols[group1] = [ Hangul_Hanja ]
    };
};
XKB

    cat > "$xkb_dir/rules/evdev" <<'RULES'
! include %S/evdev

! option = symbols
  geulbus:caps_hangul = +geulbus(caps_hangul)
  geulbus:ralt_hanja = +geulbus(ralt_hanja):all
RULES
}

# 사용법: geulbus_xkb_toggle_option <옵션이름> [--revert]
geulbus_xkb_toggle_option() {
    local option="$1" mode="${2:-}"
    local key="org.gnome.desktop.input-sources"
    local cur new
    cur="$(gsettings get "$key" xkb-options)"

    if [[ "$mode" == "--revert" ]]; then
        new="$(printf '%s' "$cur" | sed "s/'$option'//; s/, ,/,/g; s/\[, /[/; s/, \]/]/")"
        if [[ "$new" == "@as []" || "$new" == "[]" ]]; then
            gsettings reset "$key" xkb-options
        else
            gsettings set "$key" xkb-options "$new"
        fi
        # 심볼/rules 파일은 다른 geulbus 옵션이 남아 있을 수 있으므로 지우지 않는다
        # (옵션이 꺼져 있으면 파일은 무해하다).
        echo "되돌림: $option 제거. 입력 소스 전환 또는 재로그인 후 반영됩니다."
        return
    fi

    geulbus_xkb_write_files
    if [[ "$cur" == "@as []" || "$cur" == "[]" ]]; then
        gsettings set "$key" xkb-options "['$option']"
    elif [[ "$cur" != *"$option"* ]]; then
        gsettings set "$key" xkb-options "${cur%]}, '$option']"
    fi
    echo "적용값: $(gsettings get "$key" xkb-options)"
    echo "반영은 즉시(또는 입력 소스 전환 한 번)입니다. 되돌리려면: $0 --revert"
}
