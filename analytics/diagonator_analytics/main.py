import os
import sqlite3
from io import BytesIO

import matplotlib.style
import numpy as np
import pandas as pd
from flask import Flask, request, send_file
from matplotlib.dates import DateFormatter, DayLocator
from matplotlib.figure import Figure

app = Flask(__name__)

ANALYTICS_FILE = os.getenv("DIAGONATOR_ANALYTICS_FILE")


@app.route("/")
def index():
    return send_file("index.html")


@app.route("/requirement_data")
def requirement_data():
    from_date = request.args.get("from")
    to_date = request.args.get("to")
    if from_date is None:
        return "Missing 'from' parameter", 400
    elif to_date is None:
        return "Missing 'to' parameter", 400

    with sqlite3.connect(ANALYTICS_FILE) as conn:
        data = pd.read_sql_query(
            "SELECT * FROM requirement_log WHERE date >= ? AND date <= ?",
            conn,
            params=(from_date, to_date),
            parse_dates={"date": "%Y-%m-%d"},
        ).sort_values("date")
    with matplotlib.style.context("seaborn-v0_8"):
        fig = Figure()
        ax = fig.subplots()
        title = []
        for req in ("Drink 1", "Floss", "Brush teeth"):
            d = data[data["name"] == req]
            ax.plot(d["date"], d["time"] / 3600, label=req)
            title.append(f"{req}: {d["time"].median()/3600:.2f}")
        ax.set_title(", ".join(title))
        if (data["date"].max() - data["date"].min()).days <= 7:
            ax.xaxis.set_major_locator(DayLocator())
        ax.xaxis.set_major_formatter(DateFormatter("%b %d"))
        ax.legend()

    buf = BytesIO()
    fig.savefig(buf, format="png")
    buf.seek(0)
    return send_file(buf, download_name="graph.png", mimetype="image/png")


@app.route("/deactivation_data")
def deactivation_data():
    from_date = request.args.get("from")
    to_date = request.args.get("to")
    if from_date is None:
        return "Missing 'from' parameter", 400
    elif to_date is None:
        return "Missing 'to' parameter", 400

    with sqlite3.connect(ANALYTICS_FILE) as conn:
        data = pd.read_sql_query(
            "SELECT * FROM deactivate_log WHERE date >= ? AND date <= ?",
            conn,
            params=(from_date, to_date),
        )

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
        ax.set_title(f"Total: {data.shape[0]} deactivations")

    buf = BytesIO()
    fig.savefig(buf, format="png")
    buf.seek(0)
    return send_file(buf, download_name="graph.png", mimetype="image/png")
