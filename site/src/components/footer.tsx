import React from "react";
import { GithubLogo } from "./github";

export function Footer() {
    return (
        <div className="cta">
            <GithubLogo href="https://github.com/metacall/metassr" />
            <div className="ctaCopy">
                <strong>GitHub</strong>
                <span>Give us a star!</span>
            </div>
        </div>
    );
}
