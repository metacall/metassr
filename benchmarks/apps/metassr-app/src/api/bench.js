function payloadSeed(input) {
  let hash = 2166136261;
  for (let i = 0; i < input.length; i++) {
    hash ^= input.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

function GET(reqJson) {
  const req = JSON.parse(reqJson || "{}");
  const seed = payloadSeed(`${req.method}:${req.url}:${JSON.stringify(req.query || {})}`);

  return JSON.stringify({
    status: 200,
    body: {
      ok: true,
      seed,
      method: req.method,
      queryKeys: Object.keys(req.query || {}).length,
    },
  });
}

function POST(reqJson) {
  const req = JSON.parse(reqJson || "{}");
  const body = req.body ? JSON.parse(req.body) : {};

  return JSON.stringify({
    status: 200,
    body: {
      ok: true,
      seed: payloadSeed(JSON.stringify(body)),
      body,
    },
  });
}

module.exports = { GET, POST };
