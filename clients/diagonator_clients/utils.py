import datetime
import os

SERVER_URL = os.getenv("DIAGONATOR_SERVER_URL", "http://localhost:3000")
ANALYTICS_FILE = os.getenv("DIAGONATOR_ANALYTICS_FILE")


def get_datetime_pair():
    """Returns a tuple of the form (date, time) representing the current local time, where
    date is a string of the form YYYY-MM-DD, and time is the number of seconds since midnight"""
    now = datetime.datetime.now()
    midnight = now.replace(hour=0, minute=0, second=0, microsecond=0)
    return (now.strftime("%Y-%m-%d"), (now - midnight).seconds)
