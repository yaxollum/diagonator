#!/usr/bin/env python3

import json
import socket
import subprocess
import sys

SOCKET_PATH = "/tmp/diagonator-server.sock"

DMENU_CMD = ["dmenu"] + sys.argv[1:]


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
    info = send_request(server_socket, {"type": "GetInfo"})
    if info["type"] == "Info":
        requirements = [
            req for req in info["info"]["requirements"] if not req["complete"]
        ]
        if len(requirements) != 0:
            choice = (
                subprocess.run(
                    DMENU_CMD,
                    input=("\n".join(req["name"] for req in requirements)).encode(),
                    capture_output=True,
                )
                .stdout.decode()
                .strip("\n")
            )
            try:
                choice_req = next(req for req in requirements if req["name"] == choice)
                res = send_request(
                    server_socket,
                    {"type": "CompleteRequirement", "id": choice_req["id"]},
                )
                if res["type"] == "Success":
                    print(f"Successfully completed requirement: {choice}")
                else:
                    print(res)
            except StopIteration:
                print(f"Requirement with name '{choice}' not found.")
        else:
            print("No incomplete requirements.")
    else:
        print(info)
