import React, { useState, ReactNode } from 'react';

export default function Article({ title }: { title: string }) {
    let [counter, setCounter] = useState(0);

    return (
        <div>
            <div className="text-4xl font-bold">This is a cool article</div>
            <div>Article's title: {title}</div>
        </div>
    )

}