# RUN: curl -X GET http://localhost:3000/api/rbgreet
def GET(req_string)
  require 'json'
  JSON.generate({
    status: 200,
    body: { message: "Hello from Ruby!" }
  })
end

# RUN: curl -X POST http://localhost:3000/api/rbgreet -H "Content-Type: application/json" -d '{"name": "Fahd"}'
def POST(req_string)
  require 'json'
  req = JSON.parse(req_string)
  data = JSON.parse(req['body'])
  name = data['name']

  JSON.generate({
    status: 200,
    body: { message: "Hello, #{name}! From Ruby" }
  })
end
