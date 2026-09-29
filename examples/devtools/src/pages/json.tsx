import React, { useState } from 'react';
import { ToolShell } from '../components/ToolShell';

const SAMPLE_JSON = `{"name":"devtools","loaders":["node","typescript","python"],"stars":42}`;

export default function JsonTool() {
    const [input, setInput] = useState(SAMPLE_JSON);
    const [action, setAction] = useState('format');
    const [output, setOutput] = useState('');
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);

    async function run() {
        setBusy(true);
        setError('');
        setOutput('');
        try {
            const res = await fetch('/api/jsontools', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action, input }),
            });
            const payload = await res.json();
            if (payload.ok) {
                setOutput(payload.output);
                if (payload.message) {
                    setOutput(payload.message);
                }
            } else {
                setError(payload.error || 'Something went wrong');
            }
        } catch {
            setError('Request failed');
        } finally {
            setBusy(false);
        }
    }

    return (
        <ToolShell
            title="JSON tools"
            lead="Format, minify, and validate JSON documents. Runs on the Python loader."
            loader="Python · /api/jsontools"
        >
            <div className="field">
                <label htmlFor="json-input">Input</label>
                <textarea
                    id="json-input"
                    className="toolInput"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                />
            </div>
            <div className="toolRow">
                <div className="field">
                    <label htmlFor="json-action">Action</label>
                    <select
                        id="json-action"
                        className="toolSelect"
                        value={action}
                        onChange={(e) => setAction(e.target.value)}
                    >
                        <option value="format">Format</option>
                        <option value="minify">Minify</option>
                        <option value="validate">Validate</option>
                    </select>
                </div>
                <button className="button" onClick={run} disabled={busy}>
                    {busy ? 'Running…' : 'Run'}
                </button>
            </div>
            {error && <p className="toolStatus isError">{error}</p>}
            {output && (
                <div className="field">
                    <label htmlFor="json-output">Output</label>
                    <textarea id="json-output" className="toolInput" readOnly value={output} />
                </div>
            )}
        </ToolShell>
    );
}
