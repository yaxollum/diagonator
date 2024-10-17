#!/usr/bin/env python3


import operator
import sqlite3
import subprocess
import sys

import requests

from .utils import ANALYTICS_FILE, SERVER_URL, get_datetime_pair


def log_to_db(choice):
    with sqlite3.connect(ANALYTICS_FILE) as conn:
        conn.execute("""CREATE TABLE IF NOT EXISTS requirement_log(
                date TEXT NOT NULL,
                time INTEGER NOT NULL,
                name TEXT NOT NULL) STRICT""")
        conn.execute(
            "INSERT INTO requirement_log(date,time,name) VALUES (?, ?, ?)",
            get_datetime_pair() + (choice,),
        )


DMENU_CMD = ["dmenu"] + sys.argv[1:]

info = requests.post(SERVER_URL, json={"type": "GetInfo"}).json()
if info["type"] == "Info":
    requirements = [req for req in info["info"]["requirements"] if not req["complete"]]
    # sort requirements by time in ascending order
    requirements.sort(key=operator.itemgetter("due"))
    if len(requirements) != 0:
        choice = (
            subprocess.run(
                DMENU_CMD,
                input=("\n".join(req["name"] for req in requirements)).encode(),
                capture_output=True,
            )
            .stdout.decode()
            .strip("\n")
        )
        try:
            choice_req = next(req for req in requirements if req["name"] == choice)
            res = requests.post(
                SERVER_URL,
                json={"type": "CompleteRequirement", "id": choice_req["id"]},
            ).json()
            if res["type"] == "Success":
                print(f"Successfully completed requirement: {choice}")
                log_to_db(choice)
            else:
                print(res)
        except StopIteration:
            print(f"Requirement with name '{choice}' not found.")
    else:
        print("No incomplete requirements.")
else:
    print(info)
