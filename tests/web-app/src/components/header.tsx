import React, { useState } from 'react';
export function Header() {
    const [counter, setCounter] = useState(0)
    return (
        <div>
            <div>This is a header</div>
            <p>Hello from header!</p>
            <button onClick={() => setCounter(counter + 1)}>this is a counter from header {counter}</button>
        </div>
    )

}





