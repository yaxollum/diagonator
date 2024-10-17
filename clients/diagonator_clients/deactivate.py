#!/usr/bin/env python3

import sys

import requests

from .utils import SERVER_URL

if len(sys.argv) != 2:
    sys.exit("Please specify the deactivation duration in seconds.")

duration = int(sys.argv[1])

print(requests.post(SERVER_URL, json={"type": "Deactivate", "duration": duration}).text)
