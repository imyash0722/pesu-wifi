# bash completion for pesu-wifi

_pesu_wifi_completions() {
    local cur prev commands
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    commands="status start stop restart login logout select use wifi add del list daemon version help"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") )
        return 0
    fi

    case "${prev}" in
        login|select|use|del)
            local accounts=""
            if command -v pesu-wifi &>/dev/null; then
                accounts=$(pesu-wifi __list-accounts 2>/dev/null)
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
