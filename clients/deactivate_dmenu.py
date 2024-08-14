#!/usr/bin/env python3

import datetime
import subprocess
import sys

import requests

SERVER_URL = "http://localhost:3000"

if len(sys.argv) < 2:
    sys.exit("Please specify the deactivation duration in seconds.")

duration = int(sys.argv[1])

DMENU_CMD = ["dmenu"] + sys.argv[2:]


def correct():
    def round_up(t):
        h = t.hour
        if h < 4:
            h += 24
        m = t.minute
        if m < 30:
            m = 30
        else:
            h += 1
            m = 0
        return f"{h:02}:{m:02}"

    answer = (
        subprocess.run(
            DMENU_CMD,
            input="",
            capture_output=True,
        )
        .stdout.decode()
        .strip("\n")
    )
    t = datetime.datetime.now()
    t2 = t - datetime.timedelta(minutes=1)
    return answer in (round_up(t), round_up(t2))


if correct():
    print(
        requests.post(
            SERVER_URL, json={"type": "Deactivate", "duration": duration}
        ).text
    )
else:
    print("Incorrect answer.")
