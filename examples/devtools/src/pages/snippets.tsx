import React, { useEffect, useState } from 'react';
import { ToolShell } from '../components/ToolShell';

type Snippet = {
    id: string;
    title: string;
    code: string;
    language: string;
    createdAt: string;
};

export default function SnippetsTool() {
    const [snippets, setSnippets] = useState<Snippet[]>([]);
    const [loaded, setLoaded] = useState(false);
    const [title, setTitle] = useState('');
    const [language, setLanguage] = useState('bash');
    const [code, setCode] = useState('');
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);

    useEffect(() => {
        fetch('/api/snippets')
            .then((res) => res.json())
            .then((payload) => {
                setSnippets(payload.snippets || []);
                setLoaded(true);
            })
            .catch(() => setLoaded(true));
    }, []);

    async function addSnippet() {
        setBusy(true);
        setError('');
        try {
            const res = await fetch('/api/snippets', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ title, language, code }),
            });
            const payload = await res.json();
            if (payload.ok) {
                setSnippets([payload.snippet, ...snippets]);
                setTitle('');
                setCode('');
            } else {
                setError(payload.error || 'Could not save snippet');
            }
        } catch {
            setError('Request failed');
        } finally {
            setBusy(false);
        }
    }

    async function deleteSnippet(id: string) {
        try {
            const res = await fetch('/api/snippets', {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ id }),
            });
            const payload = await res.json();
            if (payload.ok) {
                setSnippets(snippets.filter((s) => s.id !== id));
            }
        } catch {
            setError('Request failed');
        }
    }

    async function copySnippet(text: string) {
        await navigator.clipboard.writeText(text);
    }

    return (
        <ToolShell
            title="Snippet box"
            lead="Store, copy, and delete reusable code snippets. Runs on the Node.js loader, kept in memory."
            loader="Node.js · /api/snippets"
        >
            <div className="field">
                <label htmlFor="sn-title">Title</label>
                <input
                    id="sn-title"
                    className="toolInput"
                    placeholder="git squash commits"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                />
            </div>
            <div className="toolRow">
                <div className="field">
                    <label htmlFor="sn-language">Language</label>
                    <select
                        id="sn-language"
                        className="toolSelect"
                        value={language}
                        onChange={(e) => setLanguage(e.target.value)}
                    >
                        <option value="bash">bash</option>
                        <option value="js">javascript</option>
                        <option value="ts">typescript</option>
                        <option value="python">python</option>
                        <option value="sql">sql</option>
                        <option value="text">text</option>
                    </select>
                </div>
            </div>
            <div className="field">
                <label htmlFor="sn-code">Code</label>
                <textarea
                    id="sn-code"
                    className="toolInput"
                    placeholder="git rebase -i HEAD~3"
                    value={code}
                    onChange={(e) => setCode(e.target.value)}
                />
            </div>
            <div className="toolRow">
                <button className="button" onClick={addSnippet} disabled={busy || !title || !code}>
                    {busy ? 'Saving…' : 'Save snippet'}
                </button>
            </div>
            {error && <p className="toolStatus isError">{error}</p>}
            <div className="snippetList">
                {loaded && snippets.length === 0 && (
                    <p className="snippetEmpty">No snippets yet — add one above.</p>
                )}
                {snippets.map((snippet) => (
                    <div key={snippet.id} className="snippet">
                        <div className="snippetMeta">
                            <h2 className="snippetTitle">{snippet.title}</h2>
                            <span className="pill">{snippet.language}</span>
                            <button className="textButton" onClick={() => copySnippet(snippet.code)}>
                                Copy
                            </button>
                            <button className="textButton" onClick={() => deleteSnippet(snippet.id)}>
                                Delete
                            </button>
                        </div>
                        <pre>{snippet.code}</pre>
                    </div>
                ))}
            </div>
        </ToolShell>
    );
}
