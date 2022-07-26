import io
import json
import socket
import sys

SOCKET_PATH = "/tmp/diagonator-server.sock"

data = """{"type": "StartSession"}"""
with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
    server_socket.connect(SOCKET_PATH)
    server_socket.sendall(data.encode() + b"\n")
    response = b""
    while b"\n" not in response:
        response += server_socket.recv(1024)
    print(response)
