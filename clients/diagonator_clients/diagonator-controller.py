import subprocess
import sys

import socketio

from .utils import SERVER_URL

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
                proc.wait()
                proc = None
