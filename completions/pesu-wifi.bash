# bash completion for pesu-wifi

_pesu_wifi_completions() {
    local cur prev commands
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    commands="status start stop restart login logout select use wifi add del list daemon help"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
        return 0
    fi

    case "${prev}" in
        login|select|use|del)
            local accounts=""
            if [[ -f "$HOME/.config/pesu-wifi/config.json" ]]; then
                accounts=$(python3 -c 'import json, os; print(" ".join(json.load(open(os.path.expanduser("~/.config/pesu-wifi/config.json"))).get("accounts", {}).keys()))' 2>/dev/null)
            fi
            COMPREPLY=( $(compgen -W "${accounts}" -- "${cur}") )
            return 0
            ;;
        wifi)
            local ssids=""
            if command -v nmcli &>/dev/null; then
                ssids=$(nmcli -t -f SSID dev wifi list 2>/dev/null | grep -v '^$' | sort -u)
            fi
            COMPREPLY=( $(compgen -W "${ssids}" -- "${cur}") )
            return 0
            ;;
        list)
            COMPREPLY=( $(compgen -W "-p --passwords" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _pesu_wifi_completions pesu-wifi
