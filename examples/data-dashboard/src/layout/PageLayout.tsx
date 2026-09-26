import { Header, Footer } from "../components/header";
import React from "react";
import { ChildrenProps } from "../types";

export function PageLayout({ children }: ChildrenProps) {
    return (
        <div className="site dashboard">
            <Header />
            <main className="dashboardMain">{children}</main>
            <Footer />
        </div>
    );
}