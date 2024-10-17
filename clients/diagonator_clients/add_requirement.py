#!/usr/bin/env python3

import sys

import requests

from .utils import SERVER_URL

if len(sys.argv) != 3:
    sys.exit(
        "Please specify the name and time of the requirement that you want to add."
    )

req_name = sys.argv[1]
req_time = sys.argv[2]

print(
    requests.post(
        SERVER_URL, json={"type": "AddRequirement", "name": req_name, "due": req_time}
    ).text
)
