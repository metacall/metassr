import React, { useState, useEffect } from 'react';

export function Clock() {
    const [time, setTime] = useState(new Date());

    useEffect(() => {
        const tick = () => setTime(new Date());

        const intervalId = setInterval(tick, 1000);

        return () => clearInterval(intervalId);
    }, []);

    // Format time as HH:MM:SS
    const formatTime = (date) => {
        const hours = date.getHours().toString().padStart(2, '0');
        const minutes = date.getMinutes().toString().padStart(2, '0');
        const seconds = date.getSeconds().toString().padStart(2, '0');
        return `${hours}:${minutes}:${seconds}`;
    };

    return (
        <div style={styles.clock}>
            {formatTime(time)}
        </div>
    );
};

const styles = {
    clock: {
        fontFamily: 'Arial, sans-serif',
        fontSize: '2em',
        textAlign: 'center',
        margin: '20px',
    },
};

export default Clock;
