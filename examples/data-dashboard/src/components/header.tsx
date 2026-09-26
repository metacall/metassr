import React from "react";

export function Header() {
    return (
        <header className="dashboardHeader">
            <p className="eyebrow">Data analysis &middot; polyglot backend</p>
            <h1 className="heroTitle">Sales &amp; Revenue Dashboard</h1>
            <p className="heroLead">
                NumPy computes the analytics in Python. Node renders the Open
                Graph image server-side. Same seeded dataset, both runtimes.
            </p>
        </header>
    );
}

export function Footer() {
    return (
        <footer className="dashboardFooter">
            <a className="cta" href="https://metassr.dev">Built with MetaSSR</a>
        </footer>
    );
}