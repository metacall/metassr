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


def GET(_req):
    return json.dumps({
        "status": 200,
        "body": {
            "service": "JSON formatter / minifier / validator",
            "language": "Python",
            "routes": {
                "POST /api/jsontools": {
                    "action": "format | minify | validate",
                    "input": "string",
                },
            },
        },
    })


def POST(req):
    try:
        data = parse_request(req)
        action = data.get("action", "format")
        text = data.get("input", "")
        parsed = json.loads(text)
        if action == "minify":
            output = json.dumps(parsed, ensure_ascii=False, separators=(",", ":"))
        else:
            output = json.dumps(parsed, indent=2, ensure_ascii=False)
        if action == "validate":
            return json.dumps({
                "status": 200,
                "body": {"ok": True, "message": "Valid JSON", "output": output},
            })
        return json.dumps({"status": 200, "body": {"ok": True, "output": output}})
    except Exception as exc:
        return json.dumps({"status": 400, "body": {"ok": False, "error": str(exc)}})
