import React, { ReactNode } from 'react';
import { renderToString } from 'react-dom/server';
import { PageLayout } from './layout/PageLayout';
import "./styles/global.css";


export default function App({ Component }) {
	return (
		<>
			<PageLayout>
				<Component />
			</PageLayout>
		</>
	);

}