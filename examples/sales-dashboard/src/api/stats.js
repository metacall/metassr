function GET(_req) {
    return JSON.stringify({
        status: 200,
        body: {
            message: "Hello from JavaScript!",
            language: "JavaScript",
            runtime: "MetaCall Node.js loader",
            timestamp: new Date().toISOString()
        }
    });
}

function POST(req) {
    var reqObj = typeof req === 'string' ? JSON.parse(req) : req;
    var data = reqObj.body ? JSON.parse(reqObj.body) : {};
    name = data.name || "anonymous";
    return JSON.stringify({
        status: 201,
        body: {
            message: "Hello, " + name + "!",
            language: "JavaScript",
            received: data,
            timestamp: new Date().toISOString()
        }
    });
}

module.exports = { GET: GET, POST: POST };
