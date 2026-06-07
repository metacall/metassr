import React from 'react';
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
