# Random Users App

This MetaSSR example shows backend randomization through an API route.

- `static/users.json` contains 50 static users.
- `src/api/users.js` reads that file and returns 10 randomly selected users.
- `src/pages/index.tsx` fetches `/api/users` when the page loads, so refreshing the browser requests a new backend-selected set.

Run it from this directory:

```bash
npm install
npm run dev
```
