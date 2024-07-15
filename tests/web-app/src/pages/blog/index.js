import React, { useState, ReactNode } from 'react';

export default function Blog() {
    let [counter, setCounter] = useState(0);

    return (
        <div>
            <div className="text-4xl font-bold">This is a cool blog</div>
        </div>
    )

}