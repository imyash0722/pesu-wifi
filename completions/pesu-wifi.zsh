#compdef pesu-wifi

_pesu_wifi() {
    local -a commands
    commands=(
        'status:Show live connection status card'
        'start:Start background keepalive daemon'
        'stop:Stop background keepalive daemon'
        'restart:Restart background keepalive daemon'
        'login:Smart login or login as specific user'
        'logout:Clean captive portal sign-out'
        'select:Set default active account'
        'use:Set default active account'
        'wifi:Select & connect to a Wi-Fi network'
        'add:Save or update login credentials'
        'del:Remove a saved account'
        'list:List saved accounts'
        'daemon:Run keepalive watchdog in foreground'
        'version:Show version information'
        'help:Show help message'
    )

    if (( CURRENT == 2 )); then
        _describe -t commands 'pesu-wifi command' commands
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
            _values 'options' '-p[Show passwords]' '--passwords[Show passwords]'
            ;;
    esac
}

_pesu_wifi "$@"
