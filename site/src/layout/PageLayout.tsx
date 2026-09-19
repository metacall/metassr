import React from "react";
import { ChildrenProps } from "../types";

export function PageLayout({ children }: ChildrenProps) {
    return <div className="site">{children}</div>;
}
