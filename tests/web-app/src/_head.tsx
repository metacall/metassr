import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';

export function __Head__() {
    return renderToString(
        <head>
            <meta charset="UTF-8"/>
            <meta name="description" content="Free Web tutorials"/>
            <meta name="keywords" content="HTML, CSS, JavaScript"/>
            <meta name="author" content="John Doe"/>
            <title>My website</title>
        </head>
    );
}


