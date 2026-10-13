import React from "react";
import { Header } from "../components/header";
import { Footer } from "../components/footer";
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
