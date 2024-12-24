import base64
import os
import sqlite3
from io import BytesIO

import matplotlib.style
import numpy as np
import pandas as pd
from flask import Flask
from matplotlib.figure import Figure

app = Flask(__name__)

ANALYTICS_FILE = os.getenv("DIAGONATOR_ANALYTICS_FILE")


@app.route("/")
def hello():
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
    encoded = base64.b64encode(buf.getbuffer()).decode("ascii")
    return f"<img src='data:image/png;base64,{encoded}'/>"
