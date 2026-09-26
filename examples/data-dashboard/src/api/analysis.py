import json

import numpy as np

PRODUCTS = [
    ("Widget", "Electronics", 480),
    ("Gadget", "Electronics", 320),
    ("Doohickey", "Hardware", 210),
    ("Contraption", "Hardware", 150),
    ("Thingamajig", "Accessories", 90),
    ("Blender", "Electronics", 260),
    ("Gear", "Hardware", 120),
    ("Cable", "Accessories", 40),
    ("Monitor", "Electronics", 540),
    ("Screw", "Hardware", 15),
    ("Sticker", "Accessories", 8),
    ("Powerbank", "Electronics", 380),
]

REGIONS = ["North", "South", "East", "West"]
MONTH_NAMES = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
]

N_RECORDS = 400
LCG_A = 1664525
LCG_C = 1013904223
LCG_M = 2 ** 32


def generate_records():
    state = 42
    seq = []
    for _ in range(N_RECORDS):
        state = (LCG_A * state + LCG_C) % LCG_M
        seq.append(state)
        state = (LCG_A * state + LCG_C) % LCG_M
        seq.append(state)
    seq = np.array(seq, dtype=np.int64)
    rec = seq[0::2]
    day_val = seq[1::2]

    names = np.array([p[0] for p in PRODUCTS], dtype=object)
    cats = np.array([p[1] for p in PRODUCTS], dtype=object)
    prices = np.array([p[2] for p in PRODUCTS], dtype=np.int64)

    product_idx = (rec % len(PRODUCTS)).astype(np.int64)
    region_idx = ((rec >> 8) % len(REGIONS)).astype(np.int64)
    units = ((rec >> 16) % 50 + 1).astype(np.int64)
    day_of_year = (day_val % 365).astype(np.int64)
    revenue = units * prices[product_idx]
    month = np.minimum(np.floor_divide(day_of_year, 30), 11)

    return {
        "product": names[product_idx],
        "category": cats[product_idx],
        "region": np.array(REGIONS, dtype=object)[region_idx],
        "units": units,
        "revenue": revenue,
        "month": month,
    }


def as_int(x):
    return int(np.round(x))


def summarize(records):
    revenue = records["revenue"]
    units = records["units"]

    total_revenue = as_int(np.sum(revenue))
    total_units = as_int(np.sum(units))
    total_orders = int(revenue.size)
    avg_order = round(total_revenue / total_orders, 2)

    monthly = np.bincount(records["month"], weights=revenue)
    half_one = as_int(np.sum(monthly[:6]))
    half_two = as_int(np.sum(monthly[6:]))
    growth = round((half_two - half_one) / half_one * 100, 1) if half_one else 0.0

    cat_sums = _group_sums(records["category"], revenue)
    top_category = max(cat_sums, key=lambda k: cat_sums[k])
    best_month = int(np.argmax(monthly))

    return {
        "totalRevenue": total_revenue,
        "totalUnits": total_units,
        "totalOrders": total_orders,
        "avgOrderValue": avg_order,
        "growthPercent": growth,
        "bestMonth": MONTH_NAMES[best_month],
        "topCategory": top_category,
    }


def _group_sums(labels, weights):
    uniq, inverse = np.unique(labels, return_inverse=True)
    sums = np.bincount(inverse, weights=weights)
    return {str(uniq[i]): as_int(sums[i]) for i in range(len(uniq))}


def monthly_revenue(records):
    monthly = np.bincount(records["month"], weights=records["revenue"])
    return [
        {"month": MONTH_NAMES[m], "revenue": as_int(monthly[m])}
        for m in range(12)
    ]


def category_revenue(records, total):
    uniq, inverse = np.unique(records["category"], return_inverse=True)
    sums = np.bincount(inverse, weights=records["revenue"])
    rows = []
    for i in range(len(uniq)):
        revenue = as_int(sums[i])
        rows.append({
            "category": str(uniq[i]),
            "revenue": revenue,
            "share": round(revenue / total * 100, 1) if total else 0.0,
        })
    return sorted(rows, key=lambda r: r["revenue"], reverse=True)


def region_revenue(records, total):
    uniq, inverse = np.unique(records["region"], return_inverse=True)
    sums = np.bincount(inverse, weights=records["revenue"])
    rows = []
    for i in range(len(uniq)):
        revenue = as_int(sums[i])
        rows.append({
            "region": str(uniq[i]),
            "revenue": revenue,
            "share": round(revenue / total * 100, 1) if total else 0.0,
        })
    return sorted(rows, key=lambda r: r["revenue"], reverse=True)


def top_products(records):
    names = records["product"]
    revenue_sums = np.zeros(len(PRODUCTS))
    unit_sums = np.zeros(len(PRODUCTS))
    for i, (name, _, _) in enumerate(PRODUCTS):
        mask = names == name
        revenue_sums[i] = np.sum(records["revenue"][mask])
        unit_sums[i] = np.sum(records["units"][mask])
    order = np.argsort(-revenue_sums)
    rows = []
    for i in order:
        name, category, _ = PRODUCTS[int(i)]
        rows.append({
            "product": name,
            "category": category,
            "revenue": as_int(revenue_sums[i]),
            "units": as_int(unit_sums[i]),
        })
    return rows


def build_payload():
    records = generate_records()
    total = as_int(np.sum(records["revenue"]))
    return {
        "dataset": {
            "records": N_RECORDS,
            "products": [p[0] for p in PRODUCTS],
            "categories": sorted({p[1] for p in PRODUCTS}),
            "regions": REGIONS,
            "months": MONTH_NAMES,
        },
        "summary": summarize(records),
        "monthlyRevenue": monthly_revenue(records),
        "categoryRevenue": category_revenue(records, total),
        "regionRevenue": region_revenue(records, total),
        "topProducts": top_products(records),
    }


def GET(req):
    return json.dumps({
        "status": 200,
        "body": build_payload(),
    })


def POST(req):
    req_obj = json.loads(req) if isinstance(req, str) else req
    return json.dumps({
        "status": 200,
        "body": build_payload(),
    })