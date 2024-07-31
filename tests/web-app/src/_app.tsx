import React from 'react';
import { renderToString } from 'react-dom/server';
import { PageLayout } from './layout/PageLayout';
import "./styles/global.css";

export default function App({ Component }) {
	return (
		<>
			<script src="https://cdn.tailwindcss.com"></script>
			<PageLayout>
				<Component />
			</PageLayout>
		</>
	);

}