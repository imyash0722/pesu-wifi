#!/usr/bin/env python3
"""
Test Suite: PESU WiFi Connections & Multi-Account Authentication
Tests Wi-Fi connection and each saved credential with controlled delays,
verifying portal state and real internet throughput, finally ensuring deltatime-1 is active.
"""
import sys
import os
import time
import json
import subprocess
import requests
from datetime import datetime

PESU_CLI = "/mnt/shared/stuff/projects/pesu-wifi/pesu_wifi.py"
PORTAL_BASE = "http://192.168.254.1:8090"
TEST_URL = "http://detectportal.firefox.com/success.txt"
BACKUP_TEST_URL = "https://www.google.com/generate_204"

# Expected timing:
# 1. NetworkManager Wi-Fi reconnect test: ~5s
# 2. Per-account cycle: ~9s * 4 = ~36s
# 3. Final deltatime-1 login & verification: ~6s
# Total expected runtime: ~48-52 seconds
EXPECTED_RUNTIME_SEC = 50

def log(msg):
    now = datetime.now().strftime("[%H:%M:%S]")
    print(f"{now} {msg}", flush=True)

def test_internet() -> tuple[bool, float, str]:
    """Test actual internet connectivity and latency."""
    s = requests.Session()
    s.trust_env = False
    s.headers.update({"Connection": "close"})
    t0 = time.time()
    try:
        r = s.get(TEST_URL, timeout=3.0)
        elapsed = time.time() - t0
        if r.status_code == 200 and "success" in r.text.lower():
            return True, elapsed, "OK (200)"
    except Exception:
        pass

    # Fallback endpoint
    t0 = time.time()
    try:
        r = s.get(BACKUP_TEST_URL, timeout=3.0)
        elapsed = time.time() - t0
        if r.status_code in (200, 204):
            return True, elapsed, f"OK ({r.status_code})"
        return False, elapsed, f"HTTP {r.status_code}"
    except Exception as e:
        return False, time.time() - t0, f"Error: {type(e).__name__}"
    finally:
        s.close()

def run_cmd(cmd: list[str]) -> tuple[int, str, str]:
    res = subprocess.run(cmd, capture_output=True, text=True)
    return res.returncode, res.stdout.strip(), res.stderr.strip()

def main():
    print("=" * 65)
    print(" PESU WiFi Automated Connection & Account Test Suite")
    print(f" Expected Runtime: ~{EXPECTED_RUNTIME_SEC} seconds (Timer target: {EXPECTED_RUNTIME_SEC * 3}s)")
    print("=" * 65, flush=True)

    start_time = time.time()
    report = {
        "start_time": datetime.now().isoformat(),
        "wifi_connection_test": {},
        "accounts_tested": {},
        "final_active_account": None,
        "success": False
    }

    # 1. Temporarily pause the background daemon to avoid race conditions
    log("Pausing pesu-wifi background service during test...")
    run_cmd(["systemctl", "--user", "stop", "pesu-wifi.service"])
    time.sleep(1)

    # 2. Test Wi-Fi network layer
    log("── Phase 1: Testing Wi-Fi Connection (PESU-EC-Campus) ──")
    code, out, err = run_cmd([sys.executable, PESU_CLI, "wifi", "PESU-EC-Campus"])
    log(f"Wi-Fi Connect: {out}")
    time.sleep(2)  # Wait for DHCP / link to stabilize

    # Check portal reachability
    s = requests.Session()
    s.trust_env = False
    portal_ok = False
    try:
        r = s.get(PORTAL_BASE, timeout=2.0)
        portal_ok = (r.status_code == 200)
    except Exception:
        portal_ok = False
    s.close()

    report["wifi_connection_test"] = {
        "ssid": "PESU-EC-Campus",
        "connect_code": code,
        "portal_online": portal_ok
    }
    log(f"Portal Gateway Reachable: {'✔ YES' if portal_ok else '✖ NO'}")

    if not portal_ok:
        log("✖ Fatal: Wi-Fi connected but portal gateway is unreachable. Aborting test.")
        # Ensure deltatime-1 and service are restored
        run_cmd([sys.executable, PESU_CLI, "select", "deltatime-1"])
        run_cmd(["systemctl", "--user", "start", "pesu-wifi.service"])
        sys.exit(1)

    # 3. Read accounts to test
    config_path = os.path.expanduser("~/.config/pesu-wifi/config.json")
    with open(config_path, "r") as f:
        cfg = json.load(f)
    accounts = list(cfg.get("accounts", {}).keys())

    # We want to test all accounts, ending with deltatime-1
    # Sort so deltatime-1 is the very last one
    test_order = [u for u in accounts if u != "deltatime-1"] + ["deltatime-1"]
    log(f"Accounts to test in sequence: {test_order}")

    log("\n── Phase 2: Testing Authentication & Internet for Each Account ──")
    for i, user in enumerate(test_order, 1):
        is_last = (i == len(test_order))
        log(f"\n[{i}/{len(test_order)}] Testing Account: '{user}'...")
        acct_result = {
            "user": user,
            "login_success": False,
            "login_output": "",
            "internet_initial": False,
            "latency_initial_ms": None,
            "stability_5s_ok": False,
            "logout_ok": False
        }

        # Step A: Explicit login
        code, out, err = run_cmd([sys.executable, PESU_CLI, "login", user])
        acct_result["login_success"] = (code == 0)
        acct_result["login_output"] = out
        log(f"  Login Result: {out}")

        if code == 0:
            # Step B: Immediate internet check
            time.sleep(0.5)
            net_ok, latency, net_msg = test_internet()
            acct_result["internet_initial"] = net_ok
            acct_result["latency_initial_ms"] = round(latency * 1000, 1)
            log(f"  Internet Access: {'✔ Connected' if net_ok else '✖ Blocked'} ({net_msg}, {round(latency*1000, 1)}ms)")

            # Step C: Hold 4 seconds to test connection stability
            log("  Holding session for 4s stability check...")
            time.sleep(4)
            net_ok2, latency2, _ = test_internet()
            acct_result["stability_5s_ok"] = net_ok2
            log(f"  Stability Hold: {'✔ Stable' if net_ok2 else '✖ Dropped'}")

        report["accounts_tested"][user] = acct_result

        # Step D: Logout unless it's the final account (deltatime-1)
        if not is_last:
            log("  Logging out for next test...")
            l_code, l_out, _ = run_cmd([sys.executable, PESU_CLI, "logout"])
            acct_result["logout_ok"] = (l_code == 0)
            log(f"  Logout Result: {l_out}")
            time.sleep(3)  # portal cool-down between session transitions
        else:
            log(f"  --> Final target reached: leaving '{user}' logged in.")

    # 4. Final Verification & Cleanup
    log("\n── Phase 3: Final State Lock-in & Service Resumption ──")
    run_cmd([sys.executable, PESU_CLI, "select", "deltatime-1"])
    # Verify deltatime-1 session is indeed active
    code, out, _ = run_cmd([sys.executable, PESU_CLI, "login", "deltatime-1"])
    final_net_ok, final_lat, _ = test_internet()
    log(f"Final Account: deltatime-1")
    log(f"Final Login State: {out}")
    log(f"Final Internet: {'✔ ONLINE' if final_net_ok else '✖ OFFLINE'} ({round(final_lat*1000, 1)}ms)")

    # Resume the background watchdog
    log("Resuming background pesu-wifi.service...")
    run_cmd(["systemctl", "--user", "start", "pesu-wifi.service"])
    time.sleep(1)

    # Get final CLI status card
    _, status_card, _ = run_cmd([sys.executable, PESU_CLI, "status"])
    print("\n" + status_card + "\n")

    total_time = round(time.time() - start_time, 2)
    report["total_time_seconds"] = total_time
    report["final_active_account"] = "deltatime-1"
    report["success"] = final_net_ok

    with open("/tmp/pesu_wifi_test_report.json", "w") as f:
        json.dump(report, f, indent=2)

    print("=" * 65)
    print(f" TEST SUITE FINISHED in {total_time}s (Target was ~{EXPECTED_RUNTIME_SEC}s)")
    print(" Report saved to: /tmp/pesu_wifi_test_report.json")
    print("=" * 65, flush=True)

if __name__ == "__main__":
    main()
