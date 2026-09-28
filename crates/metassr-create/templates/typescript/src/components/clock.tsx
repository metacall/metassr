import React, { useState, useEffect } from 'react';

const SERVER_TIME_KEY = '__METASSR_SERVER_RENDERED_AT__';

function formatTime(date: Date) {
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    return `${hours}:${minutes}:${seconds}`;
}

function initialTime(): Date {
    if (typeof window !== 'undefined') {
        const pushed = (window as unknown as Record<string, string | undefined>)[SERVER_TIME_KEY];
        if (pushed) {
            return new Date(pushed);
        }
    }
    return new Date();
}

const Clock: React.FC = () => {
    // The value rendered on the server, pushed to the client and reused during
    // hydration so the first client render matches the server HTML exactly.
    const [renderedAt] = useState<Date>(initialTime);
    // Starts from the server-rendered time, then ticks on the client.
    const [time, setTime] = useState<Date>(renderedAt);

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
