#!/usr/bin/env python3
"""
generate_ios_shortcut.py
Generates Apple Shortcuts (.shortcut) files for zero-click PES University
and Amaatra Hostel captive portal authentication on iOS / iPadOS.
"""

import argparse
import plistlib
import sys
import time

def build_shortcut_dict(username="YOUR_SRN", password="YOUR_PASSWORD", gateway_url="http://192.168.254.1:8090/login.xml"):
    timestamp = str(int(time.time() * 1000))

    actions = [
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.getcontentsofurl",
            "WFWorkflowActionParameters": {
                "WFURL": gateway_url,
                "WFHTTPMethod": "POST",
                "WFHTTPHeaders": {
                    "Value": {
                        "WFDictionaryFieldValueItems": [
                            {
                                "WFItemType": 0,
                                "WFKey": {
                                    "Value": {"string": "User-Agent"},
                                    "WFSerializationType": "WFTextTokenString"
                                },
                                "WFValue": {
                                    "Value": {"string": "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)"},
                                    "WFSerializationType": "WFTextTokenString"
                                }
                            }
                        ]
                    },
                    "WFSerializationType": "WFDictionaryFieldValue"
                },
                "WFHTTPBodyType": "Form",
                "WFFormValues": {
                    "Value": {
                        "WFDictionaryFieldValueItems": [
                            {
                                "WFItemType": 0,
                                "WFKey": {"Value": {"string": "mode"}, "WFSerializationType": "WFTextTokenString"},
                                "WFValue": {"Value": {"string": "191"}, "WFSerializationType": "WFTextTokenString"}
                            },
                            {
                                "WFItemType": 0,
                                "WFKey": {"Value": {"string": "username"}, "WFSerializationType": "WFTextTokenString"},
                                "WFValue": {"Value": {"string": username}, "WFSerializationType": "WFTextTokenString"}
                            },
                            {
                                "WFItemType": 0,
                                "WFKey": {"Value": {"string": "password"}, "WFSerializationType": "WFTextTokenString"},
                                "WFValue": {"Value": {"string": password}, "WFSerializationType": "WFTextTokenString"}
                            },
                            {
                                "WFItemType": 0,
                                "WFKey": {"Value": {"string": "a"}, "WFSerializationType": "WFTextTokenString"},
                                "WFValue": {"Value": {"string": timestamp}, "WFSerializationType": "WFTextTokenString"}
                            },
                            {
                                "WFItemType": 0,
                                "WFKey": {"Value": {"string": "producttype"}, "WFSerializationType": "WFTextTokenString"},
                                "WFValue": {"Value": {"string": "0"}, "WFSerializationType": "WFTextTokenString"}
                            }
                        ]
                    },
                    "WFSerializationType": "WFDictionaryFieldValue"
                }
            }
        },
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.notification",
            "WFWorkflowActionParameters": {
                "WFNotificationActionTitle": "PESU WiFi",
                "WFNotificationActionBody": "Campus Wi-Fi Connected & Authenticated!"
            }
        }
    ]

    plist_data = {
        "WFWorkflowClientVersion": "2607.1",
        "WFWorkflowMinimumClientVersion": 900,
        "WFWorkflowIcon": {
            "WFWorkflowIconGlyphNumber": 59796,  # Wi-Fi / Radio wave glyph
            "WFWorkflowIconStartColor": 4282601983  # Purple/Cyan gradient
        },
        "WFWorkflowActions": actions
    }

    # Add interactive import questions if using template placeholders
    if username == "YOUR_SRN":
        plist_data["WFWorkflowImportQuestions"] = [
            {
                "ActionIndex": 0,
                "Category": "Parameter",
                "ParameterKey": "username",
                "Text": "Enter your PESU SRN / Username (e.g. PES1UG22CS001)"
            },
            {
                "ActionIndex": 0,
                "Category": "Parameter",
                "ParameterKey": "password",
                "Text": "Enter your Wi-Fi Password"
            }
        ]

    return plist_data

def main():
    parser = argparse.ArgumentParser(description="Generate Apple Shortcuts (.shortcut) for PESU WiFi auto-login on iOS")
    parser.add_argument("--username", "-u", default="YOUR_SRN", help="PESU SRN / Username")
    parser.add_argument("--password", "-p", default="YOUR_PASSWORD", help="Wi-Fi Password")
    parser.add_argument("--campus", "-c", choices=["rr", "ec", "amaatra"], default="rr", help="Campus location (default: rr)")
    parser.add_argument("--output", "-o", default="PESU-WiFi-Login.shortcut", help="Output .shortcut file path")
    parser.add_argument("--format", choices=["xml", "binary"], default="xml", help="Plist output format (default: xml)")

    args = parser.parse_args()

    if args.campus in ["rr", "ec"]:
        gateway_url = "http://192.168.254.1:8090/login.xml"
    elif args.campus == "amaatra":
        gateway_url = "http://172.16.1.1:8090/login.xml"
    else:
        gateway_url = "http://192.168.254.1:8090/login.xml"

    shortcut_dict = build_shortcut_dict(
        username=args.username,
        password=args.password,
        gateway_url=gateway_url
    )

    fmt = plistlib.FMT_XML if args.format == "xml" else plistlib.FMT_BINARY
    with open(args.output, "wb") as f:
        plistlib.dump(shortcut_dict, f, fmt=fmt)

    print(f"Generated iOS Shortcut: {args.output} (Target: {gateway_url})")

if __name__ == "__main__":
    main()
