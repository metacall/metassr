import React from 'react';

const TITLE = 'Sales Dashboard | MetaSSR';
const DESCRIPTION =
    'A MetaSSR polyglot dashboard: NumPy + pandas analytics in Python and Open Graph images rendered in Node.';
const IMAGE = '/static/og/dashboard.png';

export default function Head() {
    return (
        <>
            <meta charSet="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta name="description" content={DESCRIPTION} />
            <link rel="icon" type="image/png" href="/static/assets/metacall-logo.png" />
            <title>{TITLE}</title>

            <meta property="og:type" content="website" />
            <meta property="og:site_name" content="MetaSSR" />
            <meta property="og:title" content={TITLE} />
            <meta property="og:description" content={DESCRIPTION} />
            <meta property="og:image" content={IMAGE} />
            <meta property="og:image:width" content="1200" />
            <meta property="og:image:height" content="630" />

            <meta name="twitter:card" content="summary_large_image" />
            <meta name="twitter:title" content={TITLE} />
            <meta name="twitter:description" content={DESCRIPTION} />
            <meta name="twitter:image" content={IMAGE} />
        </>
    );
}
