# Random Users App

This MetaSSR example shows backend randomization through an API route.

- `static/users.db` contains 50 static users in a SQLite database.
- `src/api/users.js` reads that database and returns 10 randomly selected users with `GET /api/users`.
- The same API route supports `PUT /api/users` to edit a user and `DELETE /api/users` to remove one.
- `src/pages/index.tsx` fetches `/api/users` when the page loads, then lets you edit or delete users from the selected sample.

Run it from this directory:

```bash
npm install
npm run dev
```
