import React from 'react';

import metacallLogo from '../../static/assets/metacall-logo.png';
import metassrLogo from '../../static/assets/metassr-logo.png';
import { Link } from '../components/link';

export default function Index() {
	return (
		<main className="page">
			<article className="heroCard">
				<div className="logoFrame">
					<img className="brandLogo" src={metassrLogo} alt="MetaSSR" />
				</div>

				<h1 className="heroTitle">
					Polyglot Programming on the Web, powered by{' '}
					<Link className="brandLink" href="https://github.com/metacall/core">
						<img className="brandMark" src={metacallLogo} alt="" />
						MetaCall
					</Link>
				</h1>

				<p className="heroLead">
					MetaSSR is a powerful experimental Server-Side Rendering (SSR) framework
					crafted for high-performance, dynamic web applications. MetaSSR uses the
					MetaCall Runtime, exploring web-based use cases for polyglot programming.
				</p>
			</article>
		</main>
	);
}
