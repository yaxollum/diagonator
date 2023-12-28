#!/usr/bin/env python3

import datetime
import json
import socket
import subprocess
import sys

SOCKET_PATH = "/tmp/diagonator-server.sock"

if len(sys.argv) < 2:
    sys.exit("Please specify the deactivation duration in seconds.")

duration = int(sys.argv[1])

DMENU_CMD = ["dmenu"] + sys.argv[2:]


def send_request(server_socket, request):
    server_socket.sendall(
        json.dumps(request).encode() + b"\n"
    )  # note that the newline ("\n") is required in order to mark the end of the request
    response = b""
    while b"\n" not in response:
        response += server_socket.recv(1024)
    return json.loads(response)


def correct():
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
    return answer in (t.strftime("%H:%M"), t2.strftime("%H:%M"))


with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
    server_socket.connect(SOCKET_PATH)
    if correct():
        print(
            send_request(
                server_socket,
                {"type": "Deactivate", "duration": duration},
            )
        )
    else:
        print("Incorrect answer.")
