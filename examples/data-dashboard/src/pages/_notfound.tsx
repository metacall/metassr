import React from 'react';

export default function NotFound() {
    return (
        <div className="dashboardNotFound">
            <p className="eyebrow">404</p>
            <h2 className="sectionTitle">Page not found</h2>
            <a href="/" className="button">Back to the dashboard</a>
        </div>
    );
}