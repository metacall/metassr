import { Footer } from "../components/footer";
import React from "react";
import { ChildrenProps } from "../types";
import { Header } from "../components/header";

export function PageLayout({ children }: ChildrenProps) {
    return (
        <div className=".container">
            <Header />
            {children}
            <Footer />
        </div>
    )
}