import React, { AnchorHTMLAttributes, ReactNode } from 'react';



export function Link({ children, href, ...args }) {
    return (
        <a target="_blank" rel="noopener noreferrer" href={href} {...args}>
            {children}
        </a>
    );
};
