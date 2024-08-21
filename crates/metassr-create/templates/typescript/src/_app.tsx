import React, { ReactNode } from 'react';
import { renderToString } from 'react-dom/server';
import { PageLayout } from './layout/PageLayout';
import "./styles/global.css";
import { ChildrenProps } from './types';


export default function App({ Component }: ChildrenProps) {
	return (
		<>
			<PageLayout>
				<Component />
			</PageLayout>
		</>
	);

}