import json
from datetime import datetime, timezone

def GET(_req):
    return json.dumps({
        "status": 200,
        "body": {
            "message": "Hello from Python!",
            "language": "Python",
            "runtime": "MetaCall Python loader",
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
    })

def POST(req):
    req_obj = json.loads(req) if isinstance(req, str) else req
    data = json.loads(req_obj.get("body", "{}")) if isinstance(req_obj.get("body"), str) else req_obj.get("body", {})
    name = data.get("name", "anonymous")
    return json.dumps({
        "status": 201,
        "body": {
            "message": f"Hello, {name}!",
            "language": "Python",
            "received": data,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
    })
