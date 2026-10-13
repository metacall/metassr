# MetaSSR vs Next.js Benchmark — 2026-09-28

**Date:** 2026-09-28
**Load:** `wrk -t12 -c1000 -d30s --latency` against each endpoint
**Setup:** Both apps run in Docker containers on the same host (linux/amd64).

| App | Image | Versions |
| --- | --- | --- |
| MetaSSR | `metassr-bench-metassr` (from `docker/app.Dockerfile`) | `metassr@1.0.0-alpha.2` from npm, SSR build |
| Next.js | `metassr-bench-nextjs` (from `benchmarks/apps/nextjs-app/Dockerfile`) | `next@16.3.6` from npm, production build |

The two apps expose the same two routes:

- `/api/bench` — identical JSON API handler in both apps (same hashing logic, same
  response payload).
- `/` — the SSR homepage. Both render the same 20-item list server-side on every
  request (`export const dynamic = 'force-dynamic'` on Next.js). MetaSSR serves a
  lean 2.9KB HTML document; Next.js serves its full 11.6KB HTML + runtime assets.

## /api/bench (identical endpoint)

| Metric | MetaSSR | Next.js | Gain |
| --- | --- | --- | --- |
| Requests/sec | 100,103.87 | 1,376.68 | **73x faster** |
| Average Latency | 11.59ms | 225.13ms | **19x lower** |
| Transfer/sec | 15.85MB | 428.87KB | **37x higher** |
| Total Requests | 3,006,409 | 41,410 | **73x more** |
| Max Latency | 1.05s | 1.90s | 1.8x lower |
| Socket Errors | 0 | 137 (12 read + 125 timeouts) | Zero errors |

## / (SSR page rendering)

| Metric | MetaSSR | Next.js | Gain |
| --- | --- | --- | --- |
| Requests/sec | 206,247.56 | 422.81 | **488x faster** |
| Average Latency | 7.20ms | 745.08ms | **103x lower** |
| Transfer/sec | 597.16MB | 4.83MB | **124x higher** |
| Total Requests | 6,207,426 | 12,720 | **488x more** |
| Max Latency | 1.03s | 1.99s | 1.9x lower |
| Socket Errors | 0 | 152 timeouts | Zero errors |

## Raw wrk output

- [metassr-root.txt](./metassr-root.txt)
- [metassr-api.txt](./metassr-api.txt)
- [nextjs-root.txt](./nextjs-root.txt)
- [nextjs-api.txt](./nextjs-api.txt)