// Example API endpoint for MetaSSR
// Test with: curl -X GET http://localhost:3000/api/hello
// Test with: curl -X POST http://localhost:3000/api/hello -H "Content-Type: application/json" -d '{"name": "world"}'

function GET(_req) {
    return JSON.stringify({
        status: 200,
        body: {
            message: "Hello from MetaSSR API!",
            timestamp: new Date().toISOString()
        }
    });
}

function POST(req) {
    // MetaCall passes the request as a JSON-encoded string to Node.js,
    // so `req` is a string rather than an object. Parse it first.
    const reqObj = typeof req === 'string' ? JSON.parse(req) : req;
    const data = reqObj.body ? JSON.parse(reqObj.body) : {};
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
