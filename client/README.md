# Example client

This folder contains an example client that connects to diagonator-server and sends commands to it.

To start the client, run

```
python client.py
```

The client assumes that diagonator-server is listening on the socket `/tmp/diagonator-server.sock`. Modify the `SOCKET_PATH` constant in `client.py` to specify a different socket location.
