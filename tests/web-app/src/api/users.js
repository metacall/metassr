/// RUN: curl -X GET http://localhost:3000/api/users
function GET(req) {
  const users = [
    { id: 1, name: "Ahmed", is_broke: true },
    { id: 2, name: "Galal", is_broke: true },
    { id: 3, name: "Eyad", is_broke: true },
    { id: 4, name: "bronny", is_broke: true },
    { id: 5, name: "Youssef", is_broke: true },
    { id: 6, name: "Fahd", is_broke: false },
  ];

  return JSON.stringify({
    status: 200,
    body: {
      data: {
        users,
        users_count: users.length
      }
    }
  });
};

/// RUN: curl -X POST http://localhost:3000/api/users -H "Content-Type: application/json" -d '{"name": "Ali", "is_broke": false}'
function POST(req) {
  const data = JSON.parse(req.body);
  const { name, is_broke } = data;

  if (!name || typeof is_broke !== "boolean") {
    return JSON.stringify({
      status: 400,
      body: { error: "'name' (string) and 'is_broke' (boolean) are required" }
    });
  }

  const newUser = { id: Math.floor(Math.random() * 1000), name, is_broke };

  return JSON.stringify({
    status: 201,
    body: { message: "User added successfully", user: newUser }
  });
};

/// RUN: curl -X PUT http://localhost:3000/api/users -H "Content-Type: application/json" -d '{"id": 3, "is_broke": false}'
function PUT(req) {
  const data = JSON.parse(req.body);
  const { id, is_broke } = data;

  if (!id || typeof is_broke !== "boolean") {
    return JSON.stringify({
      status: 400,
      body: { error: "'id' and 'is_broke' are required" }
    });
  }

  return JSON.stringify({
    status: 200,
    body: { message: `Updated user #${id}`, new_status: { is_broke } }
  });
};

/// RUN: curl -X DELETE http://localhost:3000/api/users -H "Content-Type: application/json" -d '{"id": 4}'
function DELETE(req) {
  const data = JSON.parse(req.body);
  const { id } = data;

  if (!id) {
    return JSON.stringify({
      status: 400,
      body: { error: "'id' is required" }
    });
  }

  return JSON.stringify({
    status: 200,
    body: { message: `Deleted user with id ${id}` }
  });
};

module.exports = { GET, POST, PUT, DELETE };
