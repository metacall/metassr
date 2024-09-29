import React, { useState, ReactNode } from 'react';

export default function Home() {
	let [counter, setCounter] = useState(0);

	return (
		<div>
			<div className="text-4xl font-bold">This is a simple home page contains a counter</div>

			<h1 className="text-4xl font-bold">{counter}</h1>
			<button onClick={() => { setCounter(counter + 1); }}>
				Click me :D
			</button>

		</div>
	)

}


