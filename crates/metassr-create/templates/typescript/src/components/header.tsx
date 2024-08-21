import metacallLogo from "../../static/assets/metacall-logo.png"
import { Link } from "./link"
export function Header() {
    return (
        <div>
            <div>
                <Link href="https://metacall.io">
                    <img src={metacallLogo} className="logo" alt="Metacall logo" />
                </Link>
            </div>
            <div>
                <h1>MetaSSR</h1>
                <p>Server-Side Rendering Framework built with <Link href="https://github.com/metacall/core">Metacall</Link></p>
            </div>
        </div>
    )
}