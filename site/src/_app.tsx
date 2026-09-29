import type { ComponentType } from "react";
import { PageLayout } from "./layout/PageLayout";
import "./styles/global.css";

export default function App({ Component }: { Component: ComponentType }) {
	return (
		<PageLayout>
			<Component />
		</PageLayout>
	);
}
