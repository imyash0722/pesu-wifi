#compdef pesu-wifi

_pesu_wifi() {
    local -a commands
    commands=(
        'status:Show live connection status card'
        'start:Start keepalive watchdog daemon'
        'stop:Stop background keepalive watchdog'
        'login:Smart login; optionally with a specific account'
        'logout:Sign out cleanly from captive portal'
        'select:Set default active account (alias: use)'
        'use:Set default active account'
        'add:Save or update login credentials'
        'del:Remove a saved account'
        'list:List saved accounts'
    )

    if (( CURRENT == 2 )); then
        _describe -t commands 'pesu-wifi command' commands
        _values 'options' \
            '-h[Print help information]' \
            '--help[Print help information]' \
            '-v[Print version information]' \
            '--version[Print version information]'
        return
    fi

    case $words[2] in
        login|select|use|del)
            local -a accounts
            if (( $+commands[pesu-wifi] )); then
                accounts=(${(f)"$(pesu-wifi __list-accounts 2>/dev/null)"})
            fi
            _describe -t accounts 'account' accounts
            ;;
        list)
            _values 'options' \
                '-p[Show passwords]' \
                '--passwords[Show passwords]' \
                '-h[Print help]' \
                '--help[Print help]'
            ;;
        start)
            _values 'options' \
                '-f[Run in foreground]' \
                '--foreground[Run in foreground]' \
                '-h[Print help]' \
                '--help[Print help]'
            ;;
    esac
}

_pesu_wifi "$@"
