import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';

export function __App__(text: string) {
	return renderToString(
		<div>
			<script src="https://cdn.tailwindcss.com"></script>
			<h1 className="text-4xl font-bold">Hello from {text}! </h1>
		</div>
	);
}

