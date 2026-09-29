import React, { ReactNode } from 'react';

type ToolShellProps = {
    title: string;
    lead: string;
    loader: string;
    children: ReactNode;
};

export function ToolShell({ title, lead, loader, children }: ToolShellProps) {
    return (
        <main className="toolPage">
            <a className="backLink" href="/">← All tools</a>
            <p className="eyebrow">{loader}</p>
            <h1 className="heroTitle">{title}</h1>
            <p className="heroLead">{lead}</p>
            <div className="toolPanel">
                {children}
            </div>
        </main>
    );
}
