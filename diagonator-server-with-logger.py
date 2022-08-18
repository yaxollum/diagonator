#!/usr/bin/env python3

import sys
import subprocess
import time

SERVER_CMD = ["diagonator-server"]

# see https://docs.python.org/3/library/time.html#time.strftime
TIME_FORMAT_STR = "%Y-%m-%d %H:%M:%S"

if len(sys.argv) != 2:
    print("Please specify a log file path as a command-line argument.")
    exit(1)

log_file_path = sys.argv[1]

with open(log_file_path, "a") as log_file:
    with subprocess.Popen(
        SERVER_CMD, stderr=subprocess.PIPE, universal_newlines=True
    ) as proc:
        print(f"Started diagonator-server.\nLogging stderr to file {log_file_path}.")
        for line in iter(proc.stderr.readline, ""):
            time_str = time.strftime(TIME_FORMAT_STR, time.localtime())
            line = line.strip("\n")
            print(f"[{time_str}] {line}", file=log_file, flush=True)
