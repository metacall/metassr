import { Footer } from "../components/footer";
import { Header } from "../components/header";
import React from "react";

export function PageLayout({ children }) {
    return (
        <div>
            <Header />
            {children}
            <Footer />
        </div>
    )
}