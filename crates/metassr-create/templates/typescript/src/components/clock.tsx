import React, { useState, useEffect } from 'react';

const Clock: React.FC = () => {
    const [time, setTime] = useState<Date>(new Date());

    useEffect(() => {
        const tick = () => setTime(new Date());

        const intervalId = setInterval(tick, 1000);

        return () => clearInterval(intervalId);
    }, []);

    // Format time as HH:MM:SS
    const formatTime = (date: Date) => {
        const hours = date.getHours().toString().padStart(2, '0');
        const minutes = date.getMinutes().toString().padStart(2, '0');
        const seconds = date.getSeconds().toString().padStart(2, '0');
        return `${hours}:${minutes}:${seconds}`;
    };

    return (
        <span className="clock">
            {formatTime(time)}
        </span>
    );
};

export default Clock;