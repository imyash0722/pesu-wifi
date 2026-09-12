# bash completion for pesu-wifi

_pesu_wifi_completions() {
    local cur prev commands
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    commands="status start stop login logout select use add del list"
    opts="-h --help -v --version"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        if [[ "${cur}" == -* ]]; then
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        else
            COMPREPLY=( $(compgen -W "${commands} ${opts}" -- "${cur}") )
        fi
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
        list)
            COMPREPLY=( $(compgen -W "-p --passwords -h --help" -- "${cur}") )
            return 0
            ;;
        start)
            COMPREPLY=( $(compgen -W "-f --foreground -h --help" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _pesu_wifi_completions pesu-wifi
