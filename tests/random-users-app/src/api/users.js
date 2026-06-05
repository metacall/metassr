const fs = require('fs');
const path = require('path');

function readUsers() {
    const usersPath = path.join(process.cwd(), 'static', 'users.json');
    return JSON.parse(fs.readFileSync(usersPath, 'utf8'));
}

function randomUsers(users, limit) {
    const shuffled = [...users];

    for (let i = shuffled.length - 1; i > 0; i -= 1) {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }

    return shuffled.slice(0, limit);
}

function GET(_req) {
    const users = readUsers();
    const selectedUsers = randomUsers(users, 10);

    return JSON.stringify({
        status: 200,
        body: {
            total: users.length,
            count: selectedUsers.length,
            users: selectedUsers
        }
    });
}

module.exports = { GET };
