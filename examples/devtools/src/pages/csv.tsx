import React, { useState } from 'react';
import { ToolShell } from '../components/ToolShell';

const SAMPLE_CSV = `name,role,team
Ada,engineer,core
Lin,tester,qa
Grace,engineer,core
Kiran,designer,ui`;

export default function CsvTool() {
    const [input, setInput] = useState(SAMPLE_CSV);
    const [direction, setDirection] = useState('csv2json');
    const [delimiter, setDelimiter] = useState(',');
    const [output, setOutput] = useState('');
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);

    async function convert() {
        setBusy(true);
        setError('');
        setOutput('');
        try {
            const res = await fetch('/api/csvconvert', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ direction, input, delimiter }),
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

    return (
        <ToolShell
            title="CSV ⇄ JSON"
            lead="Convert comma-separated values to JSON and back, with any delimiter. Runs on the Python loader."
            loader="Python · /api/csvconvert"
        >
            <div className="field">
                <label htmlFor="csv-input">Input</label>
                <textarea
                    id="csv-input"
                    className="toolInput"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                />
            </div>
            <div className="toolRow">
                <div className="field">
                    <label htmlFor="csv-direction">Direction</label>
                    <select
                        id="csv-direction"
                        className="toolSelect"
                        value={direction}
                        onChange={(e) => setDirection(e.target.value)}
                    >
                        <option value="csv2json">CSV → JSON</option>
                        <option value="json2csv">JSON → CSV</option>
                    </select>
                </div>
                <div className="field">
                    <label htmlFor="csv-delimiter">Delimiter</label>
                    <input
                        id="csv-delimiter"
                        className="toolInput"
                        value={delimiter}
                        maxLength={1}
                        onChange={(e) => setDelimiter(e.target.value || ',')}
                    />
                </div>
                <button className="button" onClick={convert} disabled={busy}>
                    {busy ? 'Converting…' : 'Convert'}
                </button>
            </div>
            {error && <p className="toolStatus isError">{error}</p>}
            {output && (
                <div className="field">
                    <label htmlFor="csv-output">Output</label>
                    <textarea id="csv-output" className="toolInput" readOnly value={output} />
                </div>
            )}
        </ToolShell>
    );
}
