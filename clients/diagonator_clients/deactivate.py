#!/usr/bin/env python3

import sys

from .utils import send_request

if len(sys.argv) != 2:
    sys.exit("Please specify the deactivation duration in seconds.")

duration = int(sys.argv[1])

print(send_request({"type": "Deactivate", "duration": duration}))
