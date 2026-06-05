import React, { useEffect, useState } from 'react';

type User = {
    id: number;
    name: string;
    email: string;
    role: string;
    city: string;
};

type UsersResponse = {
    total: number;
    count: number;
    users: User[];
};

export default function Index() {
    const [data, setData] = useState<UsersResponse | null>(null);
    const [error, setError] = useState('');

    useEffect(() => {
        fetch('/api/users')
            .then((response) => {
                if (!response.ok) {
                    throw new Error(`Request failed with ${response.status}`);
                }
                return response.json();
            })
            .then((payload: UsersResponse) => {
                setData(payload);
                setError('');
            })
            .catch((err: Error) => {
                setError(err.message);
            });
    }, []);

    return (
        <main className="page">
            <section className="intro">
                <p className="eyebrow">Backend randomization example</p>
                <h1>Random Users</h1>
                <p>
                    This page requests <code>/api/users</code> on load. The API handler reads
                    50 users from <code>static/users.json</code> and returns a random set of 10.
                </p>
            </section>

            {error ? <p className="error">Unable to load users: {error}</p> : null}

            <section className="panel" aria-live="polite">
                <div className="panelHeader">
                    <h2>Selected users</h2>
                    <span>{data ? `${data.count} of ${data.total}` : 'Loading'}</span>
                </div>

                <ul className="userGrid">
                    {(data?.users ?? []).map((user) => (
                        <li className="userCard" key={user.id}>
                            <div>
                                <strong>{user.name}</strong>
                                <small>{user.email}</small>
                            </div>
                            <dl>
                                <div>
                                    <dt>Role</dt>
                                    <dd>{user.role}</dd>
                                </div>
                                <div>
                                    <dt>City</dt>
                                    <dd>{user.city}</dd>
                                </div>
                            </dl>
                        </li>
                    ))}
                </ul>
            </section>
        </main>
    );
}
