import React, { useState } from 'react';
import metassrLogo from "../../static/assets/metassr-logo.svg"
import metacallLogo from "../../static/assets/metacall-logo.png"

import { Link } from '../components/link';
import Clock from '../components/clock';
import { Footer } from '../components/footer';

export default function Index() {
	const [count, setCount] = useState(0)

	return (
		<main className="page">
			<img className="logo" src={metassrLogo} alt="MetaSSR" />

			<h1 className="heroTitle">
				Server-Side Rendering Framework built with{' '}
				<Link className="brandLink" href="https://github.com/metacall/core">
					<img className="brandMark" src={metacallLogo} alt="" />
					MetaCall
				</Link>
			</h1>

			<p className="heroLead">
				Your new MetaSSR app is running. Edit <code>src/pages/index.tsx</code> and the
				server will hot-reload.
			</p>

			<section className="demos" aria-labelledby="demo-title">
				<p id="demo-title" className="eyebrow">Try it out</p>
				<div className="demo">
					<span className="demoLabel">Server-rendered clock</span>
					<Clock />
				</div>
				<div className="demo">
					<span className="demoLabel">Client-side counter</span>
					<div className="demoRow">
						<button className="button" onClick={() => setCount((count) => count + 1)}>
							Increase
						</button>
						<strong className="demoValue">{count}</strong>
					</div>
				</div>
			</section>

			<Footer />
		</main>
	)
}