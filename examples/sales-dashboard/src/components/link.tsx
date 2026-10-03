import React, { AnchorHTMLAttributes, ReactNode } from 'react';

interface LinkProps extends AnchorHTMLAttributes<HTMLAnchorElement> {
    href: string;
    children: ReactNode;
}

export const Link: React.FC<LinkProps> = ({ children, href, ...args }) => {
    return (
        <a target="_blank" rel="noopener noreferrer" href={href} {...args}>
            {children}
        </a>
    );
};
