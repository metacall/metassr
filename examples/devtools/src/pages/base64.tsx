import React, { useState } from 'react';
import { ToolShell } from '../components/ToolShell';

const SAMPLE_TEXT = 'Hello from MetaSSR!';

export default function Base64Tool() {
    const [input, setInput] = useState(SAMPLE_TEXT);
    const [mode, setMode] = useState('base64');
    const [action, setAction] = useState('encode');
    const [output, setOutput] = useState('');
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);

    async function run() {
        setBusy(true);
        setError('');
        setOutput('');
        try {
            const res = await fetch('/api/base64', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action, mode, input }),
            });
            const payload = await res.json();
            if (payload.ok) {
                setOutput(payload.output);
            } else {
                setError(payload.error || 'Something went wrong');
            }
        } catch {
            setError('Request failed');
        } finally {
            setBusy(false);
        }
    }

    async function copy() {
        if (output) {
            await navigator.clipboard.writeText(output);
        }
    }

    return (
        <ToolShell
            title="Base64 · Hex · URL"
            lead="Encode and decode text in base64, hex, or URL encoding. Runs on the Node.js loader."
            loader="Node.js · /api/base64"
        >
            <div className="field">
                <label htmlFor="b64-input">Input</label>
                <textarea
                    id="b64-input"
                    className="toolInput"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                />
            </div>
            <div className="toolRow">
                <div className="field">
                    <label htmlFor="b64-mode">Encoding</label>
                    <select
                        id="b64-mode"
                        className="toolSelect"
                        value={mode}
                        onChange={(e) => setMode(e.target.value)}
                    >
                        <option value="base64">Base64</option>
                        <option value="hex">Hex</option>
                        <option value="url">URL</option>
                    </select>
                </div>
                <div className="field">
                    <label htmlFor="b64-action">Action</label>
                    <select
                        id="b64-action"
                        className="toolSelect"
                        value={action}
                        onChange={(e) => setAction(e.target.value)}
                    >
                        <option value="encode">Encode</option>
                        <option value="decode">Decode</option>
                    </select>
                </div>
                <button className="button" onClick={run} disabled={busy}>
                    {busy ? 'Running…' : 'Run'}
                </button>
            </div>
            {error && <p className="toolStatus isError">{error}</p>}
            {output && (
                <div className="field">
                    <label htmlFor="b64-output">
                        Output{' '}
                        <button className="textButton" onClick={copy}>Copy</button>
                    </label>
                    <textarea id="b64-output" className="toolInput" readOnly value={output} />
                </div>
            )}
        </ToolShell>
    );
}
