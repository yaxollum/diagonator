#!/usr/bin/env python3

import json
import socket
import time

SOCKET_PATH = "/tmp/diagonator-server.sock"

request = {"type": "GetInfo"}

with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
    server_socket.connect(SOCKET_PATH)
    while True:
        server_socket.sendall(
            json.dumps(request).encode() + b"\n"
        )  # note that the newline ("\n") is required in order to mark the end of the request
        response = b""
        while b"\n" not in response:
            response += server_socket.recv(1024)
        response = json.loads(response)
        if response["type"] == "Info":
            unix_time = int(time.time())

            def fmt_remaining_time(t):
                duration = t - unix_time
                minutes = duration // 60
                return f"{minutes} minute{'s'[:minutes!=1]} remaining"

            def fmt_start_time(t):
                if t is None:
                    return "start of day"
                else:
                    return time.strftime("%H:%M", time.localtime(t))

            def fmt_end_time(t):
                if t is None:
                    return "end of day"
                else:
                    return time.strftime("%H:%M", time.localtime(t))

            info = response["info"]

            if info["state"] == "Unlockable":
                msg = "Session is unlockable"
            elif info["state"] == "Locked":
                msg = "Session is locked"
            else:
                assert info["state"] == "Unlocked"
                msg = "Session is unlocked"

            if info["reason"]["type"] == "BreakTimer":
                msg += f": {fmt_remaining_time(info['until'])}"
            else:
                msg += f" until {fmt_end_time(info['until'])}"

            if info["reason"]["type"] == "RequirementNotMet":
                req_id = info["reason"]["id"]
                req = next(req for req in info["requirements"] if req["id"] == req_id)
                msg += f" (requirement \"{req['name']}\" due at {fmt_start_time(req['due'])})"
            elif info["reason"]["type"] == "LockedTimeRange":
                ltr_id = info["reason"]["id"]
                ltr = next(
                    ltr for ltr in info["locked_time_ranges"] if ltr["id"] == ltr_id
                )
                msg += f" (locked time range from {fmt_start_time(ltr['start'])} to {fmt_end_time(ltr['end'])})"
            time_str = time.strftime("%a %Y-%m-%d %H:%M", time.localtime(unix_time))
            # flush=True is necessary when printing because i3bar uses a pipe
            # to communicate with this program, and Python's write buffer isn't
            # automatically flushed when printing to a pipe
            print(f"{msg} | {time_str} ", flush=True)
        else:
            # an error occured
            print(response, flush=True)
            while True:
                time.sleep(1)
        time.sleep(0.1)
