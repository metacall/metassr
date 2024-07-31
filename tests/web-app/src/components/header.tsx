import React, { useState } from 'react';
export function Header() {
    const [counter, setCounter] = useState(0)
    return (
        <div>
            <ul>
                <li><a href="/">index</a></li>
                <li><a href="/home">home</a></li>
                <li><a href="/blog">blog</a></li>
            </ul>
        </div>
    )

}





