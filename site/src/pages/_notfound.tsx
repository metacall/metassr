import React from 'react';

export default function NotFound() {
	return (
		<main className="page">
			<p className="eyebrow">Error 404</p>
			<h1 className="heroTitle">Page not found</h1>
			<p className="heroLead">
				The page you are looking for does not exist or has moved.
			</p>
			<a href="./" className="button">
				Back home
			</a>
		</main>
	);
}
