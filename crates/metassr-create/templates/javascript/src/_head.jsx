import React from 'react';

export default function Head() {
    return (
        <>
            <meta charSet="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <link rel="icon" type="image/png" href="/static/assets/metacall-logo.png" />
            <title> %NAME% | %VER% </title>
        </>
    );
}

