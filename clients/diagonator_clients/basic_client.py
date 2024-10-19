#!/usr/bin/env python3

import sqlite3
import sys

import requests

from .utils import ANALYTICS_FILE, SERVER_URL, get_datetime_pair


def log_event_to_table(table_name):
    with sqlite3.connect(ANALYTICS_FILE) as conn:
        conn.execute(f"""CREATE TABLE IF NOT EXISTS {table_name}(
                date TEXT NOT NULL,
                time INTEGER NOT NULL) STRICT""")
        conn.execute(
            f"INSERT INTO {table_name}(date,time) VALUES (?, ?)", get_datetime_pair()
        )


def log_unlock_event():
    log_event_to_table("unlock_log")


def log_lock_event():
    log_event_to_table("lock_log")


request_type = None
if len(sys.argv) == 2:
    if sys.argv[1] in ("UnlockTimer", "LockTimer", "GetInfo"):
        request_type = sys.argv[1]

if request_type is None:
    sys.exit("Please specify a request: UnlockTimer, LockTimer, or GetInfo.")

response = requests.post(SERVER_URL, json={"type": request_type}).json()
print(response)

if response["type"] == "Success":
    if request_type == "UnlockTimer":
        log_unlock_event()
    elif request_type == "LockTimer":
        log_lock_event()
