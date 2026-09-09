#!/usr/bin/env python3
"""
Unit tests for pesu_wifi module (offline, no network required).
"""
from __future__ import annotations

import os
import stat
import tempfile
import unittest
from unittest.mock import MagicMock, patch

import pesu_wifi


class TestPesuWifiHelpers(unittest.TestCase):
    def test_clean_message(self):
        self.assertEqual(pesu_wifi.clean_message("Login &amp; Success"), "Login & Success")
        self.assertEqual(pesu_wifi.clean_message("  Hello &quot;User&quot;  "), 'Hello "User"')
        self.assertEqual(pesu_wifi.clean_message(""), "")
        self.assertEqual(pesu_wifi.clean_message(None), "")

    def test_visual_len(self):
        plain = "PESU WiFi Status"
        colored = f"\033[1m\033[92m{plain}\033[0m"
        self.assertEqual(pesu_wifi.visual_len(plain), len(plain))
        self.assertEqual(pesu_wifi.visual_len(colored), len(plain))

    def test_is_campus_ssid(self):
        self.assertTrue(pesu_wifi.is_campus_ssid("PESU-EC-Campus"))
        self.assertTrue(pesu_wifi.is_campus_ssid("PESU-RR-Campus"))
        self.assertTrue(pesu_wifi.is_campus_ssid("PES-WIFI"))
        self.assertTrue(pesu_wifi.is_campus_ssid("pesuniversity-guest"))
        self.assertFalse(pesu_wifi.is_campus_ssid("Home_Network"))
        self.assertFalse(pesu_wifi.is_campus_ssid("Space"))
        self.assertFalse(pesu_wifi.is_campus_ssid(None))
        self.assertFalse(pesu_wifi.is_campus_ssid(""))


class TestConfigManagement(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.TemporaryDirectory()
        self.orig_xdg = os.environ.get("XDG_CONFIG_HOME")
        os.environ["XDG_CONFIG_HOME"] = self.test_dir.name

    def tearDown(self):
        if self.orig_xdg is not None:
            os.environ["XDG_CONFIG_HOME"] = self.orig_xdg
        else:
            os.environ.pop("XDG_CONFIG_HOME", None)
        self.test_dir.cleanup()

    def test_save_and_load_config_with_permissions(self):
        data = {
            "active_user": "PES2UG25CS000",
            "accounts": {
                "PES2UG25CS000": "secretPass123"
            },
            "preferred_ssid": "PESU-EC-Campus"
        }
        pesu_wifi.save_config_data(data)

        # Check loaded data
        loaded = pesu_wifi.load_config_data()
        self.assertEqual(loaded.get("active_user"), "PES2UG25CS000")
        self.assertEqual(loaded.get("accounts", {}).get("PES2UG25CS000"), "secretPass123")
        self.assertEqual(loaded.get("preferred_ssid"), "PESU-EC-Campus")

        # Check 0600 file permissions
        json_path = os.path.join(self.test_dir.name, "pesu-wifi", "config.json")
        env_path = os.path.join(self.test_dir.name, "pesu-wifi", ".env")
        self.assertTrue(os.path.isfile(json_path))
        self.assertTrue(os.path.isfile(env_path))

        json_mode = stat.S_IMODE(os.stat(json_path).st_mode)
        env_mode = stat.S_IMODE(os.stat(env_path).st_mode)
        self.assertEqual(json_mode, 0o600)
        self.assertEqual(env_mode, 0o600)


class TestPortalParsing(unittest.TestCase):
    @patch("pesu_wifi._request")
    def test_do_login_success_live_status(self, mock_request):
        mock_resp = MagicMock()
        mock_resp.text = "<requestresponse><status>LIVE</status><message><![CDATA[You are signed in]]></message></requestresponse>"
        mock_request.return_value = mock_resp

        ok, msg = pesu_wifi.do_login("testuser", "testpass")
        self.assertTrue(ok)
        self.assertIn("Signed in", msg)

    @patch("pesu_wifi._request")
    def test_do_login_invalid_credentials(self, mock_request):
        mock_resp = MagicMock()
        mock_resp.text = "<requestresponse><status>FAILED</status><message><![CDATA[The system could not log you on. Make sure your password is correct]]></message></requestresponse>"
        mock_request.return_value = mock_resp

        ok, msg = pesu_wifi.do_login("testuser", "wrongpass")
        self.assertFalse(ok)
        self.assertIn("password", msg.lower())

    @patch("pesu_wifi._request")
    def test_check_live_ack(self, mock_request):
        mock_resp = MagicMock()
        mock_resp.text = "<requestresponse><ack>ack</ack></requestresponse>"
        mock_request.return_value = mock_resp

        self.assertTrue(pesu_wifi.check_live("testuser", retry=False))

    @patch("pesu_wifi._request")
    def test_check_live_unreachable_exception(self, mock_request):
        mock_request.side_effect = Exception("Connection refused")
        self.assertFalse(pesu_wifi.check_live("testuser", retry=False))


if __name__ == "__main__":
    unittest.main()
