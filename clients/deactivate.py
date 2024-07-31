#!/usr/bin/env python3

import sys

import requests

SERVER_URL = "http://localhost:3000"

if len(sys.argv) != 2:
    sys.exit("Please specify the deactivation duration in seconds.")

duration = int(sys.argv[1])

print(requests.post(SERVER_URL, json={"type": "Deactivate", "duration": duration}).text)
