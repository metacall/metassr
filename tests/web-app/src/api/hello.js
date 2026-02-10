// Example API endpoint for MetaSSR
// Test with: curl -X GET http://localhost:3000/api/hello
// Test with: curl -X POST http://localhost:3000/api/hello -H "Content-Type: application/json" -d '{"name": "world"}'

function GET(req) {
    return JSON.stringify({
        status: 200,
        body: {
            message: "Hello from MetaSSR API!",
            timestamp: new Date().toISOString()
        }
    });
}

function POST(req) {
    const data = req.body ? JSON.parse(req.body) : {};
    const name = data.name || "anonymous";
    
    return JSON.stringify({
        status: 201,
        body: {
            message: `Hello, ${name}!`,
            received: data
        }
    });
}

module.exports = { GET, POST };
