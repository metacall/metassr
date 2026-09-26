function parseRequest(req) {
    const reqObj = typeof req === 'string' ? JSON.parse(req) : req;
    const body = reqObj.body ? JSON.parse(reqObj.body) : {};
    return body;
}

function base64Encode(value) {
    return Buffer.from(value, 'utf8').toString('base64');
}

function base64Decode(value) {
    return Buffer.from(value, 'base64').toString('utf8');
}

function hexEncode(value) {
    return Buffer.from(value, 'utf8').toString('hex');
}

function hexDecode(value) {
    return Buffer.from(value, 'hex').toString('utf8');
}

function GET(_req) {
    return JSON.stringify({
        status: 200,
        body: {
            service: 'Base64 / Hex / URL encoder and decoder',
            language: 'JavaScript',
            routes: {
                'POST /api/base64': {
                    action: 'encode | decode',
                    mode: 'base64 | hex | url',
                    input: 'string',
                },
            },
        },
    });
}

function POST(req) {
    try {
        const data = parseRequest(req);
        const action = data.action || 'encode';
        const mode = data.mode || 'base64';
        const input = data.input || '';

        let output;
        if (mode === 'hex') {
            output = action === 'encode' ? hexEncode(input) : hexDecode(input);
        } else if (mode === 'url') {
            output = action === 'encode' ? encodeURIComponent(input) : decodeURIComponent(input);
        } else {
            output = action === 'encode' ? base64Encode(input) : base64Decode(input);
        }

        return JSON.stringify({ status: 200, body: { ok: true, output } });
    } catch (e) {
        return JSON.stringify({ status: 400, body: { ok: false, error: String(e) } });
    }
}

module.exports = { GET: GET, POST: POST };
