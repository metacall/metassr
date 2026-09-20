const TITLE = "MetaSSR";
const DESCRIPTION =
    "MetaSSR is a powerful experimental Server-Side Rendering (SSR) framework crafted for high-performance, dynamic web applications.";
const IMAGE = "/static/assets/og-image.png";

export default function Head() {
    return (
        <>
            <meta charSet="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta name="description" content={DESCRIPTION} />
            <link rel="icon" type="image/svg+xml" href="/static/assets/metassr-logo.svg" />
            <title>{TITLE}</title>

            <meta property="og:type" content="website" />
            <meta property="og:site_name" content="MetaSSR" />
            <meta property="og:title" content={TITLE} />
            <meta property="og:description" content={DESCRIPTION} />
            <meta property="og:image" content={IMAGE} />
            <meta property="og:image:width" content="2400" />
            <meta property="og:image:height" content="1260" />

            <meta name="twitter:card" content="summary_large_image" />
            <meta name="twitter:title" content={TITLE} />
            <meta name="twitter:description" content={DESCRIPTION} />
            <meta name="twitter:image" content={IMAGE} />
        </>
    );
}
