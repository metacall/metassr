import React from "react";
import { ChildrenProps } from "../types";

export function PageLayout({ children }: ChildrenProps) {
    return (
        <div className="shell">
            <header className="siteHeader">
                <a className="brandLink" href="/">Dev tools</a>
                <nav className="nav">
                    <a href="/csv">CSV ⇄ JSON</a>
                    <a href="/json">JSON</a>
                    <a href="/base64">Base64</a>
                    <a href="/snippets">Snippets</a>
                </nav>
            </header>
            {children}
            <footer className="siteFooter">
                <span>Node · Python API routes · TypeScript frontend</span>
                <span>Built with MetaSSR</span>
            </footer>
        </div>
    )
}
