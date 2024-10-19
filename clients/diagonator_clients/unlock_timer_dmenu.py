import sys

from .basic_client import unlock_timer
from .utils import prompt_dmenu_time

if prompt_dmenu_time(sys.argv[1:]):
    unlock_timer()
