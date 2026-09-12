# fish completion for pesu-wifi

function __pesu_wifi_get_accounts
    if type -q pesu-wifi
        pesu-wifi __list-accounts 2>/dev/null
    end
end

complete -c pesu-wifi -f
complete -c pesu-wifi -s h -l help -d "Print help information"
complete -c pesu-wifi -s v -l version -d "Print version information"

complete -c pesu-wifi -n "__fish_use_subcommand" -a status -d "Show live connection status card"
complete -c pesu-wifi -n "__fish_use_subcommand" -a start -d "Start keepalive watchdog daemon"
complete -c pesu-wifi -n "__fish_use_subcommand" -a stop -d "Stop background keepalive watchdog"
complete -c pesu-wifi -n "__fish_use_subcommand" -a login -d "Smart login; optionally with a specific account"
complete -c pesu-wifi -n "__fish_use_subcommand" -a logout -d "Sign out cleanly from captive portal"
complete -c pesu-wifi -n "__fish_use_subcommand" -a select -d "Set default active account (alias: use)"
complete -c pesu-wifi -n "__fish_use_subcommand" -a use -d "Set default active account"
complete -c pesu-wifi -n "__fish_use_subcommand" -a add -d "Save or update login credentials"
complete -c pesu-wifi -n "__fish_use_subcommand" -a del -d "Remove a saved account"
complete -c pesu-wifi -n "__fish_use_subcommand" -a list -d "List saved accounts"

# Flag completions
complete -c pesu-wifi -n "__fish_seen_subcommand_from start" -s f -l foreground -d "Run watchdog in foreground"
complete -c pesu-wifi -n "__fish_seen_subcommand_from list" -s p -l passwords -d "Show passwords"

# Account completions for login, select, use, del
complete -c pesu-wifi -n "__fish_seen_subcommand_from login select use del" -a "(__pesu_wifi_get_accounts)"
