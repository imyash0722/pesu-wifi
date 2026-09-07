#!/usr/bin/env python3
"""
Test Suite: PESU WiFi Connections & Multi-Account Authentication
Tests Wi-Fi connection and each saved credential with controlled delays,
verifying portal state and real internet throughput, finally restoring the active account.
"""
import sys
import os
import time
import json
import subprocess
import requests
from datetime import datetime

PESU_CLI = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "pesu_wifi.py"))
PORTAL_BASE = "http://192.168.254.1:8090"
TEST_URL = "http://detectportal.firefox.com/success.txt"
BACKUP_TEST_URL = "https://www.google.com/generate_204"

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

    # 1. Read accounts to test and identify target active account
    config_path = os.path.expanduser("~/.config/pesu-wifi/config.json")
    if not os.path.isfile(config_path):
        log("✖ No configuration file found at ~/.config/pesu-wifi/config.json")
        sys.exit(1)

    with open(config_path, "r") as f:
        cfg = json.load(f)

    accounts = list(cfg.get("accounts", {}).keys())
    if not accounts:
        log("✖ No saved accounts to test. Run 'pesu-wifi add' first.")
        sys.exit(1)

    primary_account = cfg.get("active_user") or accounts[0]
    preferred_ssid = cfg.get("preferred_ssid") or "PESU-EC-Campus"

    # 2. Temporarily pause the background daemon to avoid race conditions
    log("Pausing pesu-wifi background service during test...")
    run_cmd(["systemctl", "--user", "stop", "pesu-wifi.service"])
    time.sleep(1)

    # 3. Test Wi-Fi network layer
    log(f"── Phase 1: Testing Wi-Fi Connection ({preferred_ssid}) ──")
    code, out, err = run_cmd([sys.executable, PESU_CLI, "wifi", preferred_ssid])
    log(f"Wi-Fi Connect: {out}")
    time.sleep(2)

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
        "ssid": preferred_ssid,
        "connect_code": code,
        "portal_online": portal_ok
    }
    log(f"Portal Gateway Reachable: {'✔ YES' if portal_ok else '✖ NO'}")

    if not portal_ok:
        log("✖ Fatal: Wi-Fi connected but portal gateway is unreachable. Aborting test.")
        run_cmd([sys.executable, PESU_CLI, "select", primary_account])
        run_cmd(["systemctl", "--user", "start", "pesu-wifi.service"])
        sys.exit(1)

    # Sequence accounts so the primary/active account is tested last and left active
    test_order = [u for u in accounts if u != primary_account] + [primary_account]
    log(f"Accounts to test in sequence: {len(test_order)} accounts configured")

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

        # Step D: Logout unless it's the final account
        if not is_last:
            log("  Logging out for next test...")
            l_code, l_out, _ = run_cmd([sys.executable, PESU_CLI, "logout"])
            acct_result["logout_ok"] = (l_code == 0)
            log(f"  Logout Result: {l_out}")
            time.sleep(3)
        else:
            log(f"  --> Final target reached: leaving '{user}' logged in.")

    # 4. Final Verification & Cleanup
    log("\n── Phase 3: Final State Lock-in & Service Resumption ──")
    run_cmd([sys.executable, PESU_CLI, "select", primary_account])
    code, out, _ = run_cmd([sys.executable, PESU_CLI, "login", primary_account])
    final_net_ok, final_lat, _ = test_internet()
    log(f"Final Account: {primary_account}")
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
    report["final_active_account"] = primary_account
    report["success"] = final_net_ok

    with open("/tmp/pesu_wifi_test_report.json", "w") as f:
        json.dump(report, f, indent=2)

    print("=" * 65)
    print(f" TEST SUITE FINISHED in {total_time}s (Target was ~{EXPECTED_RUNTIME_SEC}s)")
    print(" Report saved to: /tmp/pesu_wifi_test_report.json")
    print("=" * 65, flush=True)

if __name__ == "__main__":
    main()
