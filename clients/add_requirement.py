#!/usr/bin/env python3

import json
import socket
import subprocess
import sys
from datetime import datetime

SOCKET_PATH = "/tmp/diagonator-server.sock"

if len(sys.argv) != 3:
    sys.exit(
        "Please specify the name and time of the requirement that you want to add."
    )

req_name = sys.argv[1]
req_time = sys.argv[2]


def send_request(server_socket, request):
    server_socket.sendall(
        json.dumps(request).encode() + b"\n"
    )  # note that the newline ("\n") is required in order to mark the end of the request
    response = b""
    while b"\n" not in response:
        response += server_socket.recv(1024)
    return json.loads(response)


with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
    server_socket.connect(SOCKET_PATH)
    print(
        send_request(
            server_socket,
            {"type": "AddRequirement", "name": req_name, "due": req_time},
        )
    )
