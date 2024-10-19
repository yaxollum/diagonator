#!/usr/bin/env python3

import sqlite3
import sys

from .utils import ANALYTICS_FILE, get_datetime_pair, send_request


def log_event_to_table(table_name):
    with sqlite3.connect(ANALYTICS_FILE) as conn:
        conn.execute(f"""CREATE TABLE IF NOT EXISTS {table_name}(
                date TEXT NOT NULL,
                time INTEGER NOT NULL) STRICT""")
        conn.execute(
            f"INSERT INTO {table_name}(date,time) VALUES (?, ?)", get_datetime_pair()
        )


def unlock_timer():
    response = send_request({"type": "UnlockTimer"})
    print(response)
    if response["type"] == "Success":
        log_event_to_table("unlock_log")


def lock_timer():
    response = send_request({"type": "LockTimer"})
    print(response)
    if response["type"] == "Success":
        log_event_to_table("lock_log")


def get_info():
    print(send_request({"type": "GetInfo"}))


def main():
    if len(sys.argv) == 2:
        if sys.argv[1] == "UnlockTimer":
            unlock_timer()
            return
        elif sys.argv[1] == "LockTimer":
            lock_timer()
            return
        elif sys.argv[1] == "GetInfo":
            get_info()
            return
    sys.exit("Please specify a request: UnlockTimer, LockTimer, or GetInfo.")


if __name__ == "__main__":
    main()
