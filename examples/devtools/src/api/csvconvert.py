import csv
import io
import json


def parse_request(req):
    if isinstance(req, str):
        req_obj = json.loads(req)
    else:
        req_obj = req
    body = req_obj.get("body") or "{}"
    if isinstance(body, str):
        return json.loads(body)
    return body


def csv_to_json(text, delimiter):
    reader = csv.DictReader(io.StringIO(text), delimiter=delimiter)
    rows = [dict(row) for row in reader]
    return json.dumps(rows, indent=2, ensure_ascii=False)


def json_to_csv(text, delimiter):
    rows = json.loads(text)
    if isinstance(rows, dict):
        rows = [rows]
    if not isinstance(rows, list) or not all(isinstance(row, dict) for row in rows):
        raise ValueError("JSON must be an array of objects")
    fieldnames = []
    for row in rows:
        for key in row.keys():
            if key not in fieldnames:
                fieldnames.append(key)
    out = io.StringIO()
    writer = csv.DictWriter(out, fieldnames=fieldnames, delimiter=delimiter, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return out.getvalue()


def GET(_req):
    return json.dumps({
        "status": 200,
        "body": {
            "service": "CSV <-> JSON converter",
            "language": "Python",
            "routes": {
                "POST /api/csvconvert": {
                    "direction": "csv2json | json2csv",
                    "input": "string",
                    "delimiter": "single character, default ','",
                },
            },
        },
    })


def POST(req):
    try:
        data = parse_request(req)
        direction = data.get("direction", "csv2json")
        text = data.get("input", "")
        delimiter = data.get("delimiter") or ","
        if len(delimiter) > 1:
            delimiter = delimiter[0]
        if direction == "json2csv":
            output = json_to_csv(text, delimiter)
        else:
            output = csv_to_json(text, delimiter)
        return json.dumps({"status": 200, "body": {"ok": True, "output": output}})
    except Exception as exc:
        return json.dumps({"status": 400, "body": {"ok": False, "error": str(exc)}})
