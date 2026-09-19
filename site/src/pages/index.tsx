import React from 'react';

import metacallLogo from '../../static/assets/metacall-logo.png';
import metassrLogo from '../../static/assets/metassr-logo.png';
import { Footer } from '../components/footer';
import { Link } from '../components/link';
import { benchmarks } from '../data/benchmarks';

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

				<section className="benchmarks" aria-labelledby="benchmarks-title">
					<h2 id="benchmarks-title" className="sectionTitle">
						Benchmarks vs Next.js
					</h2>
					<p className="sectionLead">12 threads, 1000 connections, 30s.</p>
					<div className="tableWrap">
						<table className="dataTable">
							<thead>
								<tr>
									<th scope="col">Metric</th>
									<th scope="col">MetaSSR</th>
									<th scope="col">Next.js</th>
									<th scope="col">Gain</th>
								</tr>
							</thead>
							<tbody>
								{benchmarks.map(row => (
									<tr key={row.metric}>
										<th scope="row">{row.metric}</th>
										<td className="isAccent">{row.metassr}</td>
										<td>{row.nextjs}</td>
										<td>
											<span className="pill">{row.gain}</span>
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				</section>

				<Footer />
			</article>
		</main>
	);
}
