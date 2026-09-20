import json
from datetime import datetime, timedelta

import numpy as np
import pandas as pd

REGIONS = ["North", "South", "East", "West"]
PRODUCTS = ["Widget", "Gadget", "Doohickey", "Contraption", "Thingamajig"]
CATEGORIES = {
    "Widget": "Electronics",
    "Gadget": "Electronics",
    "Doohickey": "Hardware",
    "Contraption": "Hardware",
    "Thingamajig": "Accessories",
}


def generate_sales_frame():
    rng = np.random.default_rng(42)

    start = datetime(2026, 1, 1)
    end = datetime(2026, 12, 31)
    date_pool = np.array(
        [start + timedelta(days=i) for i in range((end - start).days + 1)],
        dtype=object,
    )

    products = rng.choice(np.array(PRODUCTS, dtype=object), size=100)
    regions = rng.choice(np.array(REGIONS, dtype=object), size=100)
    units = rng.integers(1, 51, size=100)
    prices = np.round(rng.uniform(10, 500, size=100), 2)
    dates = rng.choice(date_pool, size=100)

    frame = pd.DataFrame({
        "date": pd.to_datetime(dates),
        "region": regions,
        "product": products,
        "units": units.astype(int),
        "price": prices.astype(float),
    })
    frame["category"] = frame["product"].map(CATEGORIES)
    frame["revenue"] = (frame["units"] * frame["price"]).round(2)
    return frame


def compute_summary(frame):
    if frame.empty:
        return {
            "totalRevenue": 0,
            "avgOrder": 0,
            "totalOrders": 0,
            "totalUnits": 0,
        }

    revenue = frame["revenue"].to_numpy(dtype=float)
    units = frame["units"].to_numpy(dtype=int)
    return {
        "totalRevenue": round(float(np.sum(revenue)), 2),
        "avgOrder": round(float(np.mean(revenue)), 2),
        "totalOrders": int(len(frame)),
        "totalUnits": int(np.sum(units)),
    }


def build_charts(frame):
    if frame.empty:
        return {
            "categoryRevenue": [],
            "monthlyRevenue": [],
            "regionRevenue": [],
            "topProducts": [],
        }

    category_revenue = (
        frame.groupby("category", as_index=False)["revenue"]
        .sum()
        .sort_values("category")
    )
    category_revenue["revenue"] = category_revenue["revenue"].round(2)

    monthly = (
        frame.assign(month=frame["date"].dt.to_period("M").astype(str))
        .groupby("month", as_index=False)["revenue"]
        .sum()
        .sort_values("month")
    )
    monthly["revenue"] = monthly["revenue"].round(2)
    monthly["growthPercent"] = (
        monthly["revenue"]
        .pct_change()
        .replace([np.inf, -np.inf], 0)
        .fillna(0)
        .mul(100)
        .round(2)
    )
    monthly["rollingAverage"] = monthly["revenue"].rolling(window=3, min_periods=1).mean().round(2)

    region_revenue = (
        frame.groupby("region", as_index=False)["revenue"]
        .sum()
        .sort_values("revenue", ascending=False)
    )
    region_revenue["revenue"] = region_revenue["revenue"].round(2)

    top_products = (
        frame.groupby(["product", "category"], as_index=False)
        .agg(totalRevenue=("revenue", "sum"), totalUnits=("units", "sum"), orders=("product", "size"))
        .sort_values("totalRevenue", ascending=False)
    )
    top_products["totalRevenue"] = top_products["totalRevenue"].round(2)
    top_products["totalUnits"] = top_products["totalUnits"].astype(int)
    top_products["orders"] = top_products["orders"].astype(int)

    return {
        "categoryRevenue": category_revenue.to_dict(orient="records"),
        "monthlyRevenue": monthly.to_dict(orient="records"),
        "regionRevenue": region_revenue.to_dict(orient="records"),
        "topProducts": top_products.to_dict(orient="records"),
    }


def apply_filters(frame, body):
    filtered = frame

    region_filter = body.get("region")
    if region_filter:
        filtered = filtered[filtered["region"] == region_filter]

    date_from = body.get("date_from")
    if date_from:
        filtered = filtered[filtered["date"] >= pd.to_datetime(date_from)]

    date_to = body.get("date_to")
    if date_to:
        filtered = filtered[filtered["date"] <= pd.to_datetime(date_to)]

    return filtered


def build_payload(frame):
    return {
        "summary": compute_summary(frame),
        "graphs": build_charts(frame),
        "filters": {
            "regions": REGIONS,
            "products": PRODUCTS,
        },
    }


def GET(_req):
    frame = generate_sales_frame()
    return json.dumps({
        "status": 200,
        "body": build_payload(frame),
    })


def POST(req):
    req_obj = json.loads(req) if isinstance(req, str) else req
    body = json.loads(req_obj.get("body", "{}")) if isinstance(req_obj.get("body"), str) else req_obj.get("body", {})

    frame = generate_sales_frame()
    filtered = apply_filters(frame, body)

    return json.dumps({
        "status": 200,
        "body": build_payload(filtered),
    })
