# RUN: curl -X GET http://localhost:3000/api/rbgreet
def GET(req_string)
  require 'json'
  JSON.generate({
    status: 200,
    body: { message: "Hello from Ruby!" }
  })
end

# RUN: curl -X POST http://localhost:3000/api/rbgreet
def POST(req_string)
  require 'json'
  JSON.generate({
    status: 200,
    body: { message: "Hello from Ruby! (POST)" }
  })
end
