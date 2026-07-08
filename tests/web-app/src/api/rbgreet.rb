# Example API endpoint for MetaSSR
# Test with: curl -X GET http://localhost:3000/api/rbgreet
# Test with: curl -X POST http://localhost:3000/api/rbgreet -H "Content-Type: application/json" -d '{"name": "world"}'

require 'json'
require 'time'

def GET(_req)
  JSON.generate({
    status: 200,
    body: {
      message: "Hello from MetaSSR API!",
      timestamp: Time.now.utc.iso8601
    }
  })
end

def POST(req)
  req_obj = req.is_a?(String) ? JSON.parse(req) : req
  body = req_obj["body"]
  data = body.is_a?(String) ? JSON.parse(body) : (body || {})
  name = data["name"] || "anonymous"
  JSON.generate({
    status: 201,
    body: {
      message: "Hello, #{name}!",
      received: data
    }
  })
end
