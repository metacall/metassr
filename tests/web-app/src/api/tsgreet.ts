// Example TypeScript API endpoint for MetaSSR
// Test with: curl -X GET http://localhost:3000/api/tsgreet
// Test with: curl -X POST http://localhost:3000/api/tsgreet -H "Content-Type: application/json" -d '{"name": "world"}'

interface ApiRequest {
    body?: string | null;
}

export function GET(_req: string): string {
    return JSON.stringify({
        status: 200,
        body: {
            message: "Hello from MetaSSR TypeScript API!",
            timestamp: new Date().toISOString()
        }
    });
}

export function POST(req: string): string {
    // MetaCall passes the request as a JSON-encoded string, so parse it first.
    const reqObj: ApiRequest = typeof req === "string" ? JSON.parse(req) : req;
    const data = reqObj.body ? JSON.parse(reqObj.body) : {};
    const name: string = data.name || "anonymous";

    return JSON.stringify({
        status: 201,
        body: {
            message: `Hello, ${name}!`,
            received: data
        }
    });
}
