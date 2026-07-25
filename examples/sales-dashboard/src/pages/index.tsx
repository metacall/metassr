import React, { useEffect, useState } from 'react';

type ApiResponse = {
    message: string;
    language: string;
    runtime: string;
    timestamp: string;
};

export default function Index() {
    const [pyResp, setPyResp] = useState<ApiResponse | null>(null);
    const [jsResp, setJsResp] = useState<ApiResponse | null>(null);
    const [error, setError] = useState('');
    const [postName, setPostName] = useState('');
    const [postLang, setPostLang] = useState<'python' | 'javascript'>('python');
    const [postResp, setPostResp] = useState<string>('');

    useEffect(() => {
        Promise.all([
            fetch('/api/sales').then(r => r.json()),
            fetch('/api/stats').then(r => r.json())
        ])
            .then(([py, js]) => {
                setPyResp(py.body || py);
                setJsResp(js.body || js);
            })
            .catch((err: Error) => setError(err.message));
    }, []);

    const handlePost = (e: React.FormEvent) => {
        e.preventDefault();
        const endpoint = postLang === 'python' ? '/api/sales' : '/api/stats';
        fetch(endpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: postName || 'anonymous' })
        })
            .then(r => r.json())
            .then(data => setPostResp(JSON.stringify(data.body || data, null, 2)))
            .catch((err: Error) => setPostResp('Error: ' + err.message));
    };

    return (
        <main className="page">
            <h1>Sales Dashboard</h1>
            <p className="subtitle">MetaSSR polyglot API demo</p>

            {error && <p className="error">{error}</p>}

            <div className="cardGrid">
                <section className="card">
                    <h2>Python API <span className="badge">/api/sales</span></h2>
                    {pyResp ? (
                        <pre>{JSON.stringify(pyResp, null, 2)}</pre>
                    ) : (
                        <p className="loading">Loading&hellip;</p>
                    )}
                </section>

                <section className="card">
                    <h2>JavaScript API <span className="badge">/api/stats</span></h2>
                    {jsResp ? (
                        <pre>{JSON.stringify(jsResp, null, 2)}</pre>
                    ) : (
                        <p className="loading">Loading&hellip;</p>
                    )}
                </section>
            </div>

            <section className="card postCard">
                <h2>Try POST</h2>
                <form onSubmit={handlePost}>
                    <div className="formRow">
                        <input
                            type="text"
                            placeholder="Your name"
                            value={postName}
                            onChange={e => setPostName(e.target.value)}
                        />
                        <select value={postLang} onChange={e => setPostLang(e.target.value as 'python' | 'javascript')}>
                            <option value="python">/api/sales (Python)</option>
                            <option value="javascript">/api/stats (JavaScript)</option>
                        </select>
                        <button type="submit">Send</button>
                    </div>
                </form>
                {postResp && <pre className="postResult">{postResp}</pre>}
            </section>
        </main>
    );
}
