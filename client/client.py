import socket
import sys

SOCKET_PATH = "/tmp/diagonator-server.sock"

with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server_socket:
    server_socket.connect(SOCKET_PATH)
