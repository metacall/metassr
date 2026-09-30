import React, { useState, useEffect } from 'react';

const SERVER_TIME_KEY = '__METASSR_SERVER_RENDERED_AT__';

function formatTime(date) {
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    return `${hours}:${minutes}:${seconds}`;
}

function initialTime() {
    if (typeof window !== 'undefined' && window[SERVER_TIME_KEY]) {
        return new Date(window[SERVER_TIME_KEY]);
    }
    return new Date();
}

export function Clock() {
    // The value rendered on the server, pushed to the client and reused during
    // hydration so the first client render matches the server HTML exactly.
    const [renderedAt] = useState(initialTime);
    // Starts from the server-rendered time, then ticks on the client.
    const [time, setTime] = useState(renderedAt);

    useEffect(() => {
        const tick = () => setTime(new Date());

        const intervalId = setInterval(tick, 1000);

        return () => clearInterval(intervalId);
    }, []);

    return (
        <>
            <script
                dangerouslySetInnerHTML={{
                    __html: `window.${SERVER_TIME_KEY}=${JSON.stringify(renderedAt.toISOString())}`,
                }}
            />
            <span className="clock">{formatTime(time)}</span>
        </>
    );
};

export default Clock;
