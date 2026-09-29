let snippets = [
    {
        id: 'seed-1',
        title: 'Squash last 3 commits',
        language: 'bash',
        code: 'git rebase -i HEAD~3',
        createdAt: '2026-09-26T00:00:00.000Z'
    },
    {
        id: 'seed-2',
        title: 'Prune dangling images',
        language: 'bash',
        code: 'docker image prune -f',
        createdAt: '2026-09-26T00:00:00.000Z'
    },
    {
        id: 'seed-3',
        title: 'Serve a static folder',
        language: 'python',
        code: 'python3 -m http.server 8000',
        createdAt: '2026-09-26T00:00:00.000Z'
    }
];

function GET(_req) {
    return JSON.stringify({
        status: 200,
        body: {
            service: 'Snippet box (in-memory store)',
            language: 'JavaScript',
            snippets: snippets
        }
    });
}

function POST(req) {
    var reqObj = typeof req === 'string' ? JSON.parse(req) : req;
    var data = reqObj.body ? JSON.parse(reqObj.body) : {};
    var snippet = {
        id: 's' + Date.now().toString(36) + Math.random().toString(36).slice(2, 7),
        title: data.title || 'Untitled',
        language: data.language || 'text',
        code: data.code || '',
        createdAt: new Date().toISOString()
    };
    snippets = [snippet].concat(snippets);
    return JSON.stringify({ status: 201, body: { ok: true, snippet: snippet } });
}

function DELETE(req) {
    var reqObj = typeof req === 'string' ? JSON.parse(req) : req;
    var data = reqObj.body ? JSON.parse(reqObj.body) : {};
    snippets = snippets.filter(function (s) {
        return s.id !== data.id;
    });
    return JSON.stringify({ status: 200, body: { ok: true } });
}

module.exports = { GET: GET, POST: POST, DELETE: DELETE };
