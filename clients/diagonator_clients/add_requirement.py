import sys

from .utils import send_request

if len(sys.argv) != 3:
    sys.exit(
        "Please specify the name and time of the requirement that you want to add."
    )

req_name = sys.argv[1]
req_time = sys.argv[2]

print(send_request({"type": "AddRequirement", "name": req_name, "due": req_time}))
