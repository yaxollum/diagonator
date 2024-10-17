# Example clients

This folder contains example clients (written in Python) that connect to diagonator-server and send various requests to it.

## Installation

Install the clients using `pip` (this will automatically install the dependencies `python-socketio` and `requests`):
```bash
pip install .
```

## Usage

To run the clients, use `python -m diagonator_clients.<client-name> <arguments>`. For example, the basic client can be executed using
```bash
python -m diagonator_clients.basic_client GetInfo
```

By default, the clients assume that `diagonator-server` is listening on `http://localhost:3000`. This can be changed by setting the `$DIAGONATOR_SERVER_URL` environment variable.

Also, if you set the `$DIAGONATOR_ANALYTICS_FILE` environment variable, the `complete_requirement_dmenu` and `deactivate_dmenu` clients will create a SQLite database at that file path and log their data to it.

### basic_client

This client sends a single request, specified by a command-line argument, to the server. It supports 3 different requests: `UnlockTimer`, `LockTimer`, and `GetInfo`. After sending the request, the client reads the response and outputs it to the terminal.

You can bind `python -m diagonator_clients.basic_client UnlockTimer` and `python -m diagonator_clients.basic_client LockTimer` to custom keyboard shortcuts on your desktop environment as quick ways to start or end a session.

### complete_requirement_dmenu

This client retrieves a list of incomplete requirements from the server (using `GetInfo`), uses `dmenu` to ask you to pick a requirement from that list, and sends a `CompleteRequirement` request to mark that requirement as completed.

You can pass command-line arguments to `dmenu` by passing them directly to `complete_requirement_dmenu.py`. For example:

```
python -m diagonator_clients.complete_requirement_dmenu -sb darkgreen
```

### add_requirement

This client adds a one-time requirement by sending an `AddRequirement` request. It accepts two command-line arguments: the name of the requirement and its completion deadline (as a 24-hour clock time). For example:

```
python -m diagonator_clients.add_requirement "go outside" 20:00
```
