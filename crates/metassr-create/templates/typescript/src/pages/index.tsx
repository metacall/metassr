import React, { useState, ReactNode } from 'react';
import metacallLogo from "../../static/assets/metacall-logo.png"

import { Link } from '../components/link';
import Clock from '../components/clock';

export default function Index() {
	const [count, setCount] = useState(0)

	return (
		<div className='column'>
			<div className='column'>
				<h1>Current Time</h1>
				<Clock />
			</div>
			<div className="column">
				<div>
					<button className='button' onClick={() => setCount((count) => count + 1)}>
						Increase
					</button>
					<div>{count}</div>
				</div>
				<div className='column'>
					<h2>
						Build your web application
					</h2>
					<div className='row'>
						<div className='column'>
							<h2>Static-Site Generation</h2>
							<code>$ metassr-cli build -t ssg</code>
						</div>
						<div className='column'>
							<h2>Server-Side Rendering</h2>
							<code>$ metassr-cli build -t ssr</code>
						</div>
					</div>
				</div>
				<div className='column'>
					<h2>
						Run your web application
					</h2>

					<code>$ metassr-cli run</code>
				</div>
			</div>
		</div>
	)

}


