import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';
export default function Index() {
	return (
		<div>
			<script src="https://cdn.tailwindcss.com"></script>
			<h1 className="text-4xl font-bold">Hello from index page </h1>
		</div>
	)
}


