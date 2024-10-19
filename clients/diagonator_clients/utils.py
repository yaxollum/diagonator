import datetime
import os
import subprocess

SERVER_URL = os.getenv("DIAGONATOR_SERVER_URL", "http://localhost:3000")
ANALYTICS_FILE = os.getenv("DIAGONATOR_ANALYTICS_FILE")


def get_datetime_pair():
    """Returns a tuple of the form (date, time) representing the current local time, where
    date is a string of the form YYYY-MM-DD, and time is the number of seconds since midnight"""
    now = datetime.datetime.now()
    midnight = now.replace(hour=0, minute=0, second=0, microsecond=0)
    return (now.strftime("%Y-%m-%d"), (now - midnight).seconds)


def prompt_dmenu_time(dmenu_options: list[str]) -> bool:
    """Creates a dmenu prompt asking for the current time, rounded to the next half hour.
    Returns a bool indicating whether the user input the correct time.
    """

    def round_up(t):
        h = t.hour
        if h < 4:
            h += 24
        m = t.minute
        if m < 30:
            m = 30
        else:
            h += 1
            m = 0
        return f"{h:02}:{m:02}"

    answer = (
        subprocess.run(
            ["dmenu"] + dmenu_options,
            input="",
            capture_output=True,
        )
        .stdout.decode()
        .strip("\n")
    )
    t = datetime.datetime.now()
    t2 = t - datetime.timedelta(minutes=1)
    return answer in (round_up(t), round_up(t2))
