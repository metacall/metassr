import React from 'react';

export default function NotFound() {
    return (
        <main className="page">
            <p className="eyebrow">404</p>
            <h1 className="heroTitle">Page not found</h1>
            <p className="heroLead">The route you visited does not exist.</p>
            <a className="button" href="/">Back home</a>
        </main>
    );
}
