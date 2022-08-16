#!/usr/bin/env python3

import json
import socket
import sys

SOCKET_PATH = "/tmp/diagonator-server.sock"

request_type = None
if len(sys.argv) == 2:
    if sys.argv[1] in ("StartSession", "EndSession", "GetInfo"):
        request_type = sys.argv[1]

if request_type is None:
    sys.exit(f"Please specify a request: StartSession, EndSession, or GetInfo.")

request = {"type": request_type}

with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
    server_socket.connect(SOCKET_PATH)
    server_socket.sendall(
        json.dumps(request).encode() + b"\n"
    )  # note that the newline ("\n") is required in order to mark the end of the request
    response = b""
    while b"\n" not in response:
        response += server_socket.recv(1024)
    print(response.decode().strip())
