import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';

export default function Head() {
    return (
        <>
            <meta charSet="UTF-8" />
            <meta name="description" content="Free Web tutorials" />
            <meta name="keywords" content="HTML, CSS, JavaScript" />
            <meta name="author" content="John Doe" />
            <title>My website</title>
        </>
    );
}


