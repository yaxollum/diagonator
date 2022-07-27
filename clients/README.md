# Example clients

This folder contains example clients that connect to diagonator-server and send various requests to it. All of the clients are written in Python and can be executed using `./<name-of-client>.py`.

The clients assume that diagonator-server is listening on the socket `/tmp/diagonator-server.sock`. You can modify the `SOCKET_PATH` constant in each of them to specify a different socket location.

## basic_client.py

This client sends a single request, specified by a command-line argument, to the server. It supports 3 different requests: `StartSession`, `EndSession`, and `GetInfo`. After sending the request, the client reads the response and outputs it to the terminal. For example, to use the `GetInfo` request, run

```
./basic_client.py GetInfo
```

You can bind `basic_client.py StartSession` and `basic_client.py EndSession` to custom keyboard shortcuts on your desktop environment as quick ways to start or stop a session (`basic_client.py` needs to be in a directory on your `$PATH` for this to work).

## i3bar_client.py

`i3bar_client.py` can be used as [the statusline command for the status bar of the i3 window manager](https://i3wm.org/docs/userguide.html#status_command). 10 times every second, it sends a `GetInfo` request to the server, reads the information from the response, and updates the status bar with that information.

This client can be simply tested from the command-line:

```
./i3bar_client.py
```

or you can set it to your `status_command` in your i3 config file.