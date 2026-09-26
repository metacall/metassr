import React from 'react';

export default function Head() {
    return (
        <>
            <meta charSet="UTF-8" />
            <title>data-dashboard | Sales &amp; Revenue Analytics</title>
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta property="og:title" content="Sales & Revenue Dashboard" />
            <meta property="og:type" content="website" />
            <meta property="og:description" content="A MetaSSR data dashboard: NumPy analytics on the backend, Open Graph images rendered in Node." />
            <meta property="og:image" content="/static/og/dashboard.png" />
            <meta name="twitter:card" content="summary_large_image" />
        </>
    );
}