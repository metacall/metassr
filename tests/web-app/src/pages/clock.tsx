import React from 'react';

export default function Clock() {
	return (
		<div>
			<div className="text-4xl font-bold">Server-rendered time</div>
			<span id="server-time" suppressHydrationWarning>
				{new Date().toISOString()}
			</span>
		</div>
	)
}
