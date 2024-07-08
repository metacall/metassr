import React, { useState } from 'react';
export function Footer() {
    const [counter, setCounter] = useState(0)
    return (
        <footer>
            <div>This is a footer</div>
            <button onClick={() => setCounter(counter + 1)}>this is a counter from footer {counter}</button>
        </footer>
    );
}





