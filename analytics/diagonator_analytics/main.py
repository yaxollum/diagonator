import os
import sqlite3
from io import BytesIO

import matplotlib.style
import numpy as np
import pandas as pd
from flask import Flask, send_file
from matplotlib.figure import Figure

app = Flask(__name__)

ANALYTICS_FILE = os.getenv("DIAGONATOR_ANALYTICS_FILE")


@app.route("/")
def index():
    return send_file("index.html")


@app.route("/api")
def api():
    with sqlite3.connect(ANALYTICS_FILE) as conn:
        data = pd.read_sql_query("SELECT * from deactivate_log", conn)

    ul = data["state"] == "Unlockable"
    rm = (data["reason"] == "RequirementNotMet") & (~ul)
    bt = (data["reason"] == "BreakTimer") & (~ul)

    with matplotlib.style.context("seaborn-v0_8"):
        fig = Figure()
        ax = fig.subplots()
        ax.set_xticks(np.arange(0, 25))
        ax.hist(
            [
                data[ul]["time"] / 3600,
                data[bt]["time"] / 3600,
                data[rm]["time"] / 3600,
                data[~(rm | ul | bt)]["time"] / 3600,
            ],
            bins=np.arange(0, 25),
            stacked=True,
        )
        ax.legend(
            [
                "Unlockable",
                "Break Timer",
                "Requirement Not Met",
                "Locked Time Range",
            ]
        )

    buf = BytesIO()
    fig.savefig(buf, format="png")
    buf.seek(0)
    return send_file(buf, download_name="graph.png", mimetype="image/png")
