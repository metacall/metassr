import React from 'react';

const tools = [
    {
        href: '/csv',
        name: 'CSV ⇄ JSON',
        desc: 'Convert tabular data both ways, with custom delimiters.',
        loader: 'Python',
    },
    {
        href: '/json',
        name: 'JSON tools',
        desc: 'Format, minify, and validate JSON documents.',
        loader: 'Python',
    },
    {
        href: '/base64',
        name: 'Base64 · Hex · URL',
        desc: 'Encode and decode text in three encodings.',
        loader: 'Node.js',
    },
    {
        href: '/snippets',
        name: 'Snippet box',
        desc: 'Store, copy, and delete reusable code snippets.',
        loader: 'Node.js',
    },
];

export default function Index() {
    return (
        <main className="page">
            <p className="eyebrow">Polyglot SSR demo</p>
            <h1 className="heroTitle">Small tools, two runtimes</h1>
            <p className="heroLead">
                Every utility below is backed by an API route running on a
                different MetaSSR loader — Node.js and Python — with a
                TypeScript React frontend.
            </p>
            <div className="toolList">
                {tools.map((tool) => (
                    <a key={tool.href} className="toolRowLink" href={tool.href}>
                        <div>
                            <h2 className="sectionTitle">{tool.name}</h2>
                            <p className="sectionLead">{tool.desc}</p>
                        </div>
                        <span className="pill">{tool.loader}</span>
                    </a>
                ))}
            </div>
        </main>
    );
}
