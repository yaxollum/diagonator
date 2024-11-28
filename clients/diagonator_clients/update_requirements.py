import sqlite3

from .utils import ANALYTICS_FILE, get_datetime_pair, send_request

with sqlite3.connect(ANALYTICS_FILE) as conn:
    completed = {
        c[0]
        for c in conn.execute(
            "SELECT name FROM requirement_log WHERE date = ?",
            get_datetime_pair()[:1],
        ).fetchall()
    }
info = send_request({"type": "GetInfo"})
for req in info["info"]["requirements"]:
    if (not req["complete"]) and req["name"] in completed:
        print(send_request({"type": "CompleteRequirement", "id": req["id"]}))
