#!/usr/bin/env python3

import sqlite3
import sys

import requests

from .utils import ANALYTICS_FILE, SERVER_URL, get_datetime_pair, prompt_dmenu_time

if len(sys.argv) < 2:
    sys.exit("Please specify the deactivation duration in seconds.")

duration = int(sys.argv[1])

if prompt_dmenu_time(sys.argv[2:]):
    print(
        requests.post(
            SERVER_URL, json={"type": "Deactivate", "duration": duration}
        ).text
    )
    if ANALYTICS_FILE is not None:
        current_state = requests.post(SERVER_URL, json={"type": "GetInfo"}).json()
        with sqlite3.connect(ANALYTICS_FILE) as conn:
            conn.execute("""CREATE TABLE IF NOT EXISTS deactivate_log(
                date TEXT NOT NULL,
                time INTEGER NOT NULL,
                state TEXT NOT NULL,
                reason TEXT NOT NULL,
                details TEXT) STRICT""")
            date, time = get_datetime_pair()
            info = current_state["info"]
            state = info["state"]
            reason = info["reason"]["type"]
            if reason == "RequirementNotMet":
                req_id = info["reason"]["id"]
                req = next(r for r in info["requirements"] if r["id"] == req_id)
                details = req["name"]
            else:
                details = None
            conn.execute(
                "INSERT INTO deactivate_log(date,time,state,reason,details) VALUES (?, ?, ?, ?, ?)",
                (date, time, state, reason, details),
            )
else:
    print("Incorrect answer.")
