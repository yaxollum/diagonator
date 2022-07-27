#!/usr/bin/env python3

import io
import json
import socket
import sys
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

            def fmt_remaining_time(t):
                duration = t - int(time.time())
                return f"{duration//60}:{duration%60:02} remaining"

            state = response["info"]["state"]
            # flush=True is necessary when printing because i3bar uses a pipe
            # to communicate with this program, and Python's write buffer isn't
            # automatically flushed when printing to a pipe
            if state["type"] == "Unlockable":
                print("Session is unlockable", flush=True)
            elif state["type"] == "Locked":
                print(
                    f"Session is locked: {fmt_remaining_time(state['until'])}",
                    flush=True,
                )
            else:
                assert state["type"] == "Active"
                print(
                    f"Session is active: {fmt_remaining_time(state['until'])}",
                    flush=True,
                )
        else:
            # an error occured
            print(response, flush=True)
            while True:
                time.sleep(1)
        time.sleep(0.1)
