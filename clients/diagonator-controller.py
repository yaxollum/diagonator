#!/usr/bin/env python3

import subprocess
import sys

import socketio

SERVER_URL = "http://localhost:3000"
DIAGONATOR_CMD = ["diagonator"] + sys.argv[1:]

proc = None

with socketio.SimpleClient() as sio:
    sio.connect(SERVER_URL)
    while True:
        event = sio.receive()
        if event[0] == "info_update":
            running = event[1]["diagonator_running"]
            if running and proc is None:
                proc = subprocess.Popen(DIAGONATOR_CMD)
            elif not running and proc is not None:
                proc.terminate()
                proc = None
