/// RUN: curl -X GET http://localhost:3000/api/time
function GET(reqString) {
  const now = new Date();

  return JSON.stringify({
    status: 200,
    body: {
      timestamp: now.getTime(),
      iso: now.toISOString(),
      message: "Current server time"
    }
  });
}

/// RUN: curl -X POST http://localhost:3000/api/time -H "Content-Type: application/json" -d '{"timezone": "Africa/Cairo"}'
function POST(req) {
  const data = JSON.parse(req.body);
  const { timezone } = data;
  const now = new Date();

  return JSON.stringify({
    status: 200,
    body: {
      message: `Received request for timezone ${timezone || "UTC"}`,
      iso: now.toISOString()
    }
  });
}

/// RUN: curl -X PUT http://localhost:3000/api/time -H "Content-Type: application/json" -d '{"offset": 2}'
function PUT(req) {
  const data = JSON.parse(req.body);
  const { offset } = data;
  const now = new Date();
  const adjusted = new Date(now.getTime() + (offset || 0) * 60 * 60 * 1000);

  return JSON.stringify({
    status: 200,
    body: {
      message: `Time adjusted by ${offset || 0} hours`,
      new_time: adjusted.toISOString()
    }
  });
}

/// RUN: curl -X DELETE http://localhost:3000/api/time
function DELETE() {
  return JSON.stringify({
    status: 200,
    body: { message: "Time data cache cleared" }
  });
}

module.exports = { GET, POST, PUT, DELETE };
