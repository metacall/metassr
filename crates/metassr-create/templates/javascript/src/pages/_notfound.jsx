import React from 'react';
import { Link } from '../components/link';

export default function NotFound() {
    return (
        <div>

            <h1 className="text-4xl font-bold">404 Page not found</h1>
            <a href='/'>
                <button className="button">
                    Back Home
                </button>
            </a>
        </div>
    )
}



