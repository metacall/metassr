const path = require('path');
const { DatabaseSync } = require('node:sqlite');

const usersDbPath = path.join(process.cwd(), 'static', 'users.db');
const editableFields = ['name', 'email', 'role', 'city'];

function openDatabase() {
    return new DatabaseSync(usersDbPath);
}

function rowToUser(row) {
    return {
        id: row.id,
        name: row.name,
        email: row.email,
        role: row.role,
        city: row.city
    };
}

function parseBody(req) {
    const reqObj = typeof req === 'string' ? JSON.parse(req) : req;

    if (!reqObj.body) {
        return {};
    }

    return typeof reqObj.body === 'string' ? JSON.parse(reqObj.body) : reqObj.body;
}

function jsonResponse(status, body) {
    return JSON.stringify({ status, body });
}

function errorResponse(status, message) {
    return jsonResponse(status, { error: message });
}

function GET(_req) {
    const db = openDatabase();

    try {
        const total = db.prepare('SELECT COUNT(*) AS total FROM users').get().total;
        const users = db
            .prepare(
                `
                SELECT id, name, email, role, city
                FROM users
                ORDER BY RANDOM()
                LIMIT 10
                `
            )
            .all()
            .map(rowToUser);

        return jsonResponse(200, {
            total,
            count: users.length,
            users
        });
    } finally {
        db.close();
    }
}

function PUT(req) {
    let payload;

    try {
        payload = parseBody(req);
    } catch (_err) {
        return errorResponse(400, 'Request body must be valid JSON.');
    }

    const id = Number(payload.id);

    if (!Number.isInteger(id)) {
        return errorResponse(400, 'A numeric user id is required.');
    }

    const db = openDatabase();

    try {
        const existingUser = db
            .prepare('SELECT id, name, email, role, city FROM users WHERE id = ?')
            .get(id);

        if (!existingUser) {
            return errorResponse(404, `User ${id} was not found.`);
        }

        const updates = {};

        for (const field of editableFields) {
            if (typeof payload[field] === 'string') {
                updates[field] = payload[field].trim();
            }
        }

        const updatedUser = {
            ...rowToUser(existingUser),
            ...updates
        };

        db.prepare(
            `
            UPDATE users
            SET name = ?, email = ?, role = ?, city = ?
            WHERE id = ?
            `
        ).run(updatedUser.name, updatedUser.email, updatedUser.role, updatedUser.city, id);

        const total = db.prepare('SELECT COUNT(*) AS total FROM users').get().total;

        return jsonResponse(200, {
            total,
            user: updatedUser
        });
    } finally {
        db.close();
    }
}

function DELETE(req) {
    let payload;

    try {
        payload = parseBody(req);
    } catch (_err) {
        return errorResponse(400, 'Request body must be valid JSON.');
    }

    const id = Number(payload.id);

    if (!Number.isInteger(id)) {
        return errorResponse(400, 'A numeric user id is required.');
    }

    const db = openDatabase();

    try {
        const result = db.prepare('DELETE FROM users WHERE id = ?').run(id);

        if (result.changes === 0) {
            return errorResponse(404, `User ${id} was not found.`);
        }

        const total = db.prepare('SELECT COUNT(*) AS total FROM users').get().total;

        return jsonResponse(200, {
            total,
            deletedId: id
        });
    } finally {
        db.close();
    }
}

module.exports = { GET, PUT, DELETE };
