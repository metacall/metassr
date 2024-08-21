import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';

export default function Head() {
    return (
        <>
            <meta charSet="UTF-8" />
            <link rel="icon" type="image/png" href="/static/assets/metacall-logo.png" />
            <title> %NAME% | %VER% </title>
        </>
    );
}


