import React, { FormEvent, useEffect, useState } from 'react';

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

type UserDraft = Pick<User, 'name' | 'email' | 'role' | 'city'>;

export default function Index() {
    const [data, setData] = useState<UsersResponse | null>(null);
    const [editingId, setEditingId] = useState<number | null>(null);
    const [draft, setDraft] = useState<UserDraft>({
        name: '',
        email: '',
        role: '',
        city: ''
    });
    const [error, setError] = useState('');
    const [actionError, setActionError] = useState('');
    const [savingId, setSavingId] = useState<number | null>(null);

    const loadUsers = () => {
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
                setActionError('');
            })
            .catch((err: Error) => {
                setError(err.message);
            });
    };

    useEffect(() => {
        loadUsers();
    }, []);

    const startEditing = (user: User) => {
        setEditingId(user.id);
        setDraft({
            name: user.name,
            email: user.email,
            role: user.role,
            city: user.city
        });
        setActionError('');
    };

    const updateDraft = (field: keyof UserDraft, value: string) => {
        setDraft((currentDraft) => ({
            ...currentDraft,
            [field]: value
        }));
    };

    const saveUser = (event: FormEvent<HTMLFormElement>, id: number) => {
        event.preventDefault();
        setSavingId(id);
        setActionError('');

        fetch('/api/users', {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ id, ...draft })
        })
            .then((response) => {
                if (!response.ok) {
                    throw new Error(`Update failed with ${response.status}`);
                }

                return response.json();
            })
            .then((payload: { user: User; total: number }) => {
                setData((currentData) => {
                    if (!currentData) {
                        return currentData;
                    }

                    const users = currentData.users.map((user) =>
                        user.id === payload.user.id ? payload.user : user
                    );

                    return {
                        total: payload.total,
                        count: users.length,
                        users
                    };
                });
                setEditingId(null);
            })
            .catch((err: Error) => {
                setActionError(err.message);
            })
            .finally(() => {
                setSavingId(null);
            });
    };

    const deleteUser = (id: number) => {
        setSavingId(id);
        setActionError('');

        fetch('/api/users', {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ id })
        })
            .then((response) => {
                if (!response.ok) {
                    throw new Error(`Delete failed with ${response.status}`);
                }

                return response.json();
            })
            .then((payload: { deletedId: number; total: number }) => {
                setData((currentData) => {
                    if (!currentData) {
                        return currentData;
                    }

                    const users = currentData.users.filter((user) => user.id !== payload.deletedId);

                    return {
                        total: payload.total,
                        count: users.length,
                        users
                    };
                });

                if (editingId === id) {
                    setEditingId(null);
                }
            })
            .catch((err: Error) => {
                setActionError(err.message);
            })
            .finally(() => {
                setSavingId(null);
            });
    };

    return (
        <main className="page">
            <section className="intro">
                <p className="eyebrow">Backend randomization example</p>
                <h1>Random Users</h1>
                <p>
                    This page requests <code>/api/users</code> on load. The API handler reads
                    users from <code>static/users.db</code>, returns a random set of 10, and
                    persists edits or deletions back to SQLite.
                </p>
            </section>

            {error ? <p className="error">Unable to load users: {error}</p> : null}
            {actionError ? <p className="error">Unable to save change: {actionError}</p> : null}

            <section className="panel" aria-live="polite">
                <div className="panelHeader">
                    <h2>Selected users</h2>
                    <div className="panelActions">
                        <span>{data ? `${data.count} of ${data.total}` : 'Loading'}</span>
                        <button type="button" className="secondaryButton" onClick={loadUsers}>
                            Refresh sample
                        </button>
                    </div>
                </div>

                <ul className="userGrid">
                    {(data?.users ?? []).map((user) => {
                        const isEditing = editingId === user.id;
                        const isSaving = savingId === user.id;

                        return (
                            <li className="userCard" key={user.id}>
                                {isEditing ? (
                                    <form className="editForm" onSubmit={(event) => saveUser(event, user.id)}>
                                        <label>
                                            Name
                                            <input
                                                value={draft.name}
                                                onChange={(event) => updateDraft('name', event.target.value)}
                                            />
                                        </label>
                                        <label>
                                            Email
                                            <input
                                                type="email"
                                                value={draft.email}
                                                onChange={(event) => updateDraft('email', event.target.value)}
                                            />
                                        </label>
                                        <label>
                                            Role
                                            <input
                                                value={draft.role}
                                                onChange={(event) => updateDraft('role', event.target.value)}
                                            />
                                        </label>
                                        <label>
                                            City
                                            <input
                                                value={draft.city}
                                                onChange={(event) => updateDraft('city', event.target.value)}
                                            />
                                        </label>
                                        <div className="cardActions">
                                            <button type="submit" disabled={isSaving}>
                                                {isSaving ? 'Saving...' : 'Save'}
                                            </button>
                                            <button
                                                type="button"
                                                className="secondaryButton"
                                                onClick={() => setEditingId(null)}
                                                disabled={isSaving}
                                            >
                                                Cancel
                                            </button>
                                        </div>
                                    </form>
                                ) : (
                                    <>
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
                                        <div className="cardActions">
                                            <button type="button" onClick={() => startEditing(user)}>
                                                Edit
                                            </button>
                                            <button
                                                type="button"
                                                className="dangerButton"
                                                onClick={() => deleteUser(user.id)}
                                                disabled={isSaving}
                                            >
                                                {isSaving ? 'Deleting...' : 'Delete'}
                                            </button>
                                        </div>
                                    </>
                                )}
                            </li>
                        );
                    })}
                </ul>
            </section>
        </main>
    );
}
