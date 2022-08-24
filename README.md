# diagonator-server
A server that controls [diagonator](https://github.com/yaxollum/diagonator), giving you a break timer, daily requirements, and locked time ranges

## Installation

Install `diagonator-server` using [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```bash
git clone https://github.com/yaxollum/diagonator-server.git
cd diagonator-server
cargo install --path .
```

## Usage

Run `diagonator-server` to start the server.

This will create a configuration file with the default options. See the [Configuration](#configuration) section for specifics on configuration.

### Logging

`diagonator-server` does not write to a log file. It prints all messages to its standard error (stderr). You can log the server's stderr using [`diagonator-server-with-logger.py`](diagonator-server-with-logger.py), which prefixes each line with a timestamp before logging it to a file of your choice. Run `diagonator-server-with-logger.py` by specifying the log file's location as a command-line argument.

## Concepts

`diagonator-server` has 3 possible states and 3 concepts that determine which state it is in.

The 3 states are:

1. `Unlocked` - diagonator is not running. You can use your computer as usual.
2. `Locked` - diagonator is running. You *cannot* unlock the timer to enter the `Unlocked` state.
3. `Unlockable` - diagonator is running. You can unlock the timer to instantly enter the `Unlocked` state.

The 3 concepts are:

1. Break Timer - By default, the timer gives you a 25-minute work period during which the server is `Unlocked`, followed by a 5-minute break during which the server is `Locked`. After the break, the server enters the `Unlockable` state, where you can instantly unlock the timer to start another work period.
2. Requirements - A requirement is a task that you have to complete by a certain time before you can continue using your computer. For example, suppose you were to set a requirement called "brush teeth" that has to be completed by 20:00. At 20:00, if the server sees that "brush teeth" has already been completed, then nothing happens. Otherwise, the server is `Locked` until you complete "brush teeth".
3. Locked Time Ranges - A locked time range is a time interval during which the server is always `Locked`. For example, if you wanted to always go to bed at 23:00 and wake up at 7:00, then you could set two separate locked time intervals: one from 23:00 to the end of the day (24:00), the other from the start of the day (0:00) to 7:00.

## Configuration

The file path of the configuration file is printed in the first line of the server's output. The configuration file uses the TOML format. After you edit the configuration, restart the server to apply your changes.

Some tips to consider when customizing your configuration:

- Set `diagonator_path` to the path to your diagonator executable.

- Use `diagonator_args` to pass command-line arguments to diagonator. Specify each argument as a separate string in the list. For example, the command `diagonator --top-margin 50` would correspond to `diagonator_args = ["--top-margin", "50"]`.

- Use 24-hour clock strings with the format `"HH:MM"` (e.g. `"16:30"`) to specify clock times.

- If you don't want any requirements, remove all the entries that start with `[[requirements]]`.

- If you don't want any locked time ranges, remove all the entries that start with `[[locked_time_ranges]]`.

- If you want a locked time range to start at the beginning of the day (0:00), omit the `start` field.

- If you want a locked time range to last until the end of the day (24:00), omit the `end` field.

## Clients

`diagonator-server` listens on a UNIX domain socket. The socket path is specified as `socket_path` in the configuration file.

The [`clients`](clients) folder contains some example clients that demonstrate how to connect to the server and send various requests to it.

Requests and responses use the JSON format. Each request/response is restricted to a single line (no newlines allowed in the middle). This allows the client and server to determine the end of each message. After a client connects to the server, it can send multiple requests (see the [i3bar client](clients/i3bar_client.py) for an example of this).

The available requests are:

- `UnlockTimer` - Unlock the break timer
- `LockTimer` - Lock the break timer
- `GetInfo` - Get information on the status of the server: the current state, when the current state will change, the reason for the current state, a list of requirements, and a list of locked time ranges
- `CompleteRequirement` - Mark a requirement as completed by specifying its ID
- `AddRequirement` - Add a one-time requirement by specifying its name and completion deadline
