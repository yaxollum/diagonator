import datetime
import os
import subprocess

import pytest
import requests

SERVER_URL = os.getenv("DIAGONATOR_SERVER_URL", "http://localhost:3000")
ANALYTICS_FILE = os.getenv("DIAGONATOR_ANALYTICS_FILE")


def send_request(json) -> dict:
    """Sends a JSON request to the server and returns the decoded JSON response"""
    return requests.post(SERVER_URL, json=json).json()


def get_datetime_pair():
    """Returns a tuple of the form (date, time) representing the current local time, where
    date is a string of the form YYYY-MM-DD, and time is the number of seconds since midnight"""
    now = datetime.datetime.now()
    midnight = now.replace(hour=0, minute=0, second=0, microsecond=0)
    return (now.strftime("%Y-%m-%d"), (now - midnight).seconds)


def get_answer_impl(t, wake_up, bedtime):
    def plural(n: int | str, unit: str):
        if str(n) == "1":
            return f"1 {unit}"
        else:
            return f"{n} {unit}s"

    def time_until(t, target_hour):
        h = t.hour
        m = t.minute
        if h >= target_hour:
            h -= 24
        diff_minutes = (target_hour - h) * 60 - m
        total = diff_minutes // 30
        if total == 0:
            return plural(diff_minutes, "minute")
        elif total % 2 == 0:
            return plural(total // 2, "hour")
        else:
            return plural(str(total // 2) + ".5", "hour")

    h = t.hour
    m = t.minute
    if h < wake_up - 3 or h >= bedtime:
        return f"{time_until(t, wake_up)} until {wake_up:02}:00 - no more work"
    elif h >= bedtime - 2:
        return f"{time_until(t, bedtime)} until bedtime - no more work"
    elif h >= bedtime - 4:
        return f"{time_until(t, bedtime)} until bedtime"
    else:
        n = -(-(h * 60 + m) // 30)
        h = n // 2
        m = n % 2 * 30
        return f"{h:02}:{m:02}"


def get_answer(t):
    """Returns the answer expected by prompt_dmenu_time at time t"""
    return get_answer_impl(t, 7, 22)


def prompt_dmenu_time(dmenu_options: list[str]) -> bool:
    """Creates a dmenu prompt asking for the current time, rounded to the next half hour.
    Returns a bool indicating whether the user input the correct time.
    """

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
    return answer in (get_answer(t), get_answer(t2))


@pytest.mark.parametrize(
    "hm,expected",
    [
        ("18:30", "18:30"),
        ("18:31", "19:00"),
        ("18:59", "19:00"),
        ("19:00", "4 hours until bedtime"),
        ("19:01", "3.5 hours until bedtime"),
        ("20:59", "2 hours until bedtime"),
        ("21:00", "2 hours until bedtime - no more work"),
        ("21:01", "1.5 hours until bedtime - no more work"),
        ("22:00", "1 hour until bedtime - no more work"),
        ("22:01", "0.5 hours until bedtime - no more work"),
        ("22:30", "0.5 hours until bedtime - no more work"),
        ("22:31", "29 minutes until bedtime - no more work"),
        ("22:59", "1 minute until bedtime - no more work"),
        ("23:00", "9 hours until 08:00 - no more work"),
        ("23:01", "8.5 hours until 08:00 - no more work"),
        ("23:59", "8 hours until 08:00 - no more work"),
        ("0:00", "8 hours until 08:00 - no more work"),
        ("2:29", "5.5 hours until 08:00 - no more work"),
        ("2:42", "5 hours until 08:00 - no more work"),
        ("4:59", "3 hours until 08:00 - no more work"),
        ("5:00", "05:00"),
    ],
)
def test_get_answer(hm: str, expected: str):
    t = datetime.datetime.strptime(hm, "%H:%M")
    assert get_answer_impl(t, 8, 23) == expected
