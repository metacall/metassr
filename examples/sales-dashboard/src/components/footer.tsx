import React from "react";
import { GithubLogo } from "./github";
import { Link } from "./link";

export function Footer() {
    return (
        <Link className="cta" href="https://github.com/metacall/metassr">
            <GithubLogo />
            <span aria-hidden="true">·</span>
            <span>give us a star</span>
        </Link>
    );
}
