import React from 'react';

export default function NotFound() {
	return (
		<section className="section notFound">
			<p className="eyebrow">Error 404</p>
			<h2 className="sectionTitle">Page not found</h2>
			<p className="sectionLead">
				The page you are looking for does not exist or has moved.
			</p>
			<a href="/" className="button">
				Back home
			</a>
		</section>
	);
}
