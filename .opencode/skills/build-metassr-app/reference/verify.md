# Verify checklist

Run these after `metassr start` (SSR) or `metassr start --serve` (SSG) before
telling the user the app works. The server listens on port 8080 by default.

## SSR (`metassr build -t ssr` + `metassr start`)

```sh
curl -fsS -o /dev/null http://localhost:8080/          # 200
curl -fsS http://localhost:8080/ | grep -q "<title>"   # rendered HTML
curl -fsS -o /dev/null http://localhost:8080/home      # 200 (per page route)
curl -fsS -o /dev/null http://localhost:8080/api/hello # 200 + JSON body
curl -fsS -o /dev/null http://localhost:8080/static/... # 200 (static asset)
curl -fsS -o /dev/null http://localhost:8080/_notfound # 200 (404 page)
curl -fsS -o /dev/null http://localhost:8080/nonexistent-route # 303 (redirect)
```

Check each page route and each API route the idea called for, not just the
root.

## SSG (`metassr build -t ssg` + `metassr start --serve`)

Same as above, except:

```sh
curl -fsS -o /dev/null http://localhost:8080/nonexistent-route # 404
```

and confirm pre-rendered HTML exists per route:

```sh
test -f dist/pages/index.html
test -f dist/pages/home/index.html
```

## After edits

Re-run the checks for anything the edit touched (changed page, new route, new
API endpoint, new static file).