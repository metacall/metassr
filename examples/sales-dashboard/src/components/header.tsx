import React from "react";

export function Header() {
    return (
        <header className="dashboardHeader">
            <p className="eyebrow">NumPy + pandas &middot; Node Open Graph</p>
            <h1 className="heroTitle">Sales Dashboard</h1>
            <p className="heroLead">
                The Python route builds chart-ready aggregates from generated
                sales data. The same app renders a 1200&times;630 Open Graph
                image server-side from a Node route, pixel by pixel.
            </p>
        </header>
    );
}
