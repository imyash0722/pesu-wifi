# fish completion for pesu-wifi

function __pesu_wifi_get_accounts
    if test -f "$HOME/.config/pesu-wifi/config.json"
        python3 -c 'import json, os; print("\n".join(json.load(open(os.path.expanduser("~/.config/pesu-wifi/config.json"))).get("accounts", {}).keys()))' 2>/dev/null
    end
end

complete -c pesu-wifi -f
complete -c pesu-wifi -n "__fish_use_subcommand" -a status -d "Show live connection status"
complete -c pesu-wifi -n "__fish_use_subcommand" -a start -d "Start background keepalive daemon"
complete -c pesu-wifi -n "__fish_use_subcommand" -a stop -d "Stop background keepalive daemon"
complete -c pesu-wifi -n "__fish_use_subcommand" -a restart -d "Restart background keepalive daemon"
complete -c pesu-wifi -n "__fish_use_subcommand" -a login -d "Smart login or login as user"
complete -c pesu-wifi -n "__fish_use_subcommand" -a logout -d "Clean captive portal sign-out"
complete -c pesu-wifi -n "__fish_use_subcommand" -a select -d "Set default active account"
complete -c pesu-wifi -n "__fish_use_subcommand" -a use -d "Set default active account"
complete -c pesu-wifi -n "__fish_use_subcommand" -a wifi -d "Select & connect to Wi-Fi"
complete -c pesu-wifi -n "__fish_use_subcommand" -a add -d "Save or update login credentials"
complete -c pesu-wifi -n "__fish_use_subcommand" -a del -d "Remove a saved account"
complete -c pesu-wifi -n "__fish_use_subcommand" -a list -d "List saved accounts"
complete -c pesu-wifi -n "__fish_use_subcommand" -a daemon -d "Run keepalive daemon in foreground"
complete -c pesu-wifi -n "__fish_use_subcommand" -a help -d "Show help message"

# Account completions for login, select, use, del
complete -c pesu-wifi -n "__fish_seen_subcommand_from login select use del" -a "(__pesu_wifi_get_accounts)"
complete -c pesu-wifi -n "__fish_seen_subcommand_from list" -s p -l passwords -d "Show passwords"
