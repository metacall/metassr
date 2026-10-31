import React, { useId, useState } from 'react';

import metacallLogo from '../../static/assets/metacall-logo.png';
import metassrLogo from '../../static/assets/metassr-logo.svg';
import { Footer } from '../components/footer';
import { Link } from '../components/link';
import { apiBenchmarks, ssrBenchmarks } from '../data/benchmarks';
import type { Benchmark } from '../data/benchmarks';

function BenchmarkTable({ rows }: { rows: Benchmark[] }) {
	const noteId = useId();
	const [showNote, setShowNote] = useState(false);
	const infoRow = rows.find(row => row.info);

	return (
		<div className="tableBlock">
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
						{rows.map(row => (
							<tr key={row.metric}>
								<th scope="row">
									<span className="metricLabel">
										{row.metric}
										{row.info && (
											<button
												type="button"
												className="infoButton"
												aria-label={`What are ${row.metric}?`}
												aria-expanded={showNote}
												aria-controls={noteId}
												title={row.info}
												onClick={() => setShowNote(value => !value)}
											>
												i
											</button>
										)}
									</span>
								</th>
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
			{infoRow?.info && (
				<p className="tableNote" id={noteId} hidden={!showNote}>
					{infoRow.info}
				</p>
			)}
		</div>
	);
}

export default function Index() {
	return (
		<main className="page">
			<img className="logo" src={metassrLogo} alt="MetaSSR" />

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
				<p className="sectionLead">
					12 threads, 1000 connections, 30s · 2.5 GHz cap.
				</p>
				<h3 className="tableTitle">API route /api/bench</h3>
				<p className="tableLead">
					Equivalent JSON handler and payload in both apps.
				</p>
				<BenchmarkTable rows={apiBenchmarks} />
				<h3 className="tableTitle">SSR page /</h3>
				<p className="tableLead">
					Both apps render the same 20-item list on every request.
				</p>
				<BenchmarkTable rows={ssrBenchmarks} />
				<p className="benchmarkSource">
					MetaSSR: <Link href="https://github.com/metacall/metassr">source</Link> /{' '}
					<Link href="https://github.com/metacall/metassr/blob/master/README.md">README</Link>
					{' · '}
					Next.js: <Link href="https://github.com/vercel/next.js">source</Link> /{' '}
					<Link href="https://github.com/vercel/next.js/blob/canary/README.md">README</Link>
				</p>
			</section>

			<Footer />
		</main>
	);
}
