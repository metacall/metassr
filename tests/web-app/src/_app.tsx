import React, { useState, ReactNode } from 'react';
import { renderToString } from 'react-dom/server';
import { Header } from './components/header';
import { Footer } from './components/footer';
import { hydrateRoot, createRoot } from 'react-dom/client';
import { PageLayout } from './layout/PageLayout';
import "./styles/global.css"

export default function App({ Component }) {
	return (
		<>
			<script src="https://cdn.tailwindcss.com"></script>
			<PageLayout>
				<Component />
			</PageLayout>
		</>
	)
}