import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';

export function Home() {
	let [counter, setCounter] = useState(0);

	return (
		<div>
			<script src="https://cdn.tailwindcss.com"></script>
			<div className="text-4xl font-bold">This is a simple home page contains a conuter</div>

			<h1 className="text-4xl font-bold">{counter}</h1>
			<button onClick={() => { setCounter(counter + 1); }}>
				Click me :D
			</button>
		</div>
	)
}

