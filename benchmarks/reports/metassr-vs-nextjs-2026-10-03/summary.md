# MetaSSR vs Next.js Benchmark — 2026-10-03

> **PRE-FIX — superseded.** MetaSSR `1.0.0-alpha.2` predates the per-request SSR
> fix (#220), so the SSR row below measures boot-time cached HTML, not SSR.
> Use `../metassr-vs-nextjs-2026-10-31/` for current-source per-request numbers.

**Load:** `wrk -t12 -c1000 -d30s --latency` (headline) plus the canonical
`compare.py` scenarios (t1c10 / t4c50 / t8c200 / t12c500, 5s each).
**Method:** both apps run in Docker with `--network host` (wrk talks to the
server directly — no `docker-proxy`, no veth). One server at a time. Every
headline figure is the **median of 3 interleaved rounds** (metassr → nextjs →
metassr → …), not a single sample.

| App | Image | Versions |
| --- | --- | --- |
| MetaSSR | `metassr-bench-metassr` (from `docker/app.Dockerfile`) | `metassr@1.0.0-alpha.2` from npm, SSR build |
| Next.js | `metassr-bench-nextjs` (from `benchmarks/apps/nextjs-app/Dockerfile`) | `next@16.3.7` from npm, production build |

Apps: `benchmarks/apps/metassr-app` and `benchmarks/apps/nextjs-app`. Both
expose `/` (the same 20-item list, server-rendered per request) and
`/api/bench` (identical JSON handler and payload).

Host: 13th Gen Intel Core i7-13650HX, 20 cores, 15.3 GB RAM, Linux 6.18.

> **Why the previous run (2026-09-28) was invalid:** it published ports with
> `-p` and hit `localhost:PORT`, so all traffic went through Docker's
> single-threaded userland proxy (`docker-proxy`). Next.js was measured at
> 422 RPS there; with host networking the same app measures 825 RPS (SSR) /
> 2,615 RPS (API). That report is kept as
> `metassr-vs-nextjs-2026-09-28-INVALID-docker-proxy/`.

## Headline — `/api/bench` (identical endpoint, median of 3)

| Metric | MetaSSR | Next.js | Gain |
| --- | --- | --- | --- |
| Requests/sec | 162,601.16 | 2,615.09 | **62x faster** |
| Average Latency | 6.43ms | 98.71ms | **15x lower** |
| Transfer/sec | 25.74MB | 814.67KB | **32x higher** |
| Total Requests | 4,894,277 | 78,681 | **62x more** |
| Max Latency | 216.85ms | 1.98s | **9.1x lower** |
| Socket Errors | 0 | 294 timeouts | **Zero errors** |

## Headline — `/` (SSR page, median of 3)

| Metric | MetaSSR | Next.js | Gain |
| --- | --- | --- | --- |
| Requests/sec | 505,105.72 | 824.88 | **612x faster** |
| Average Latency | 2.05ms | 383.29ms | **187x lower** |
| Transfer/sec | 1.43GB | 9.42MB | **155x higher** |
| Total Requests | 15,196,308 | 24,824 | **612x more** |
| Max Latency | 240.72ms | 1.99s | **8.3x lower** |
| Socket Errors | 0 | 304 timeouts | **Zero errors** |

Note: the two SSR responses differ in size — MetaSSR serves a lean 2.9KB HTML
document, Next.js serves 11.6KB of HTML plus its runtime/chunk references.
The API row is the byte-for-byte identical comparison.

## Canonical scenarios (single pass, 5s each)

| Scenario | Endpoint | MetaSSR RPS | Next.js RPS | Gain |
| --- | --- | --- | --- | --- |
| Light (t1 c10) | `/` | 146,794 | 757 | 194x |
| Medium (t4 c50) | `/` | 459,809 | 784 | 587x |
| Heavy (t8 c200) | `/` | 598,270 | 764 | 783x |
| Stress (t12 c500) | `/` | 652,926 | 780 | 837x |
| Light (t1 c10) | `/api/bench` | 141,387 | 2,718 | 52x |
| Medium (t4 c50) | `/api/bench` | 224,103 | 2,719 | 82x |
| Heavy (t8 c200) | `/api/bench` | 230,190 | 2,617 | 88x |
| Stress (t12 c500) | `/api/bench` | 195,654 | 2,457 | 80x |

MetaSSR served the canonical scenarios with **0 socket errors**; Next.js
logged timeouts at Heavy and Stress (61–72 per run).

## Run-to-run stability (headline)

| Framework | Endpoint | Round 1 | Round 2 | Round 3 | Spread |
| --- | --- | --- | --- | --- | --- |
| MetaSSR | `/` | 561,233 | 500,394 | 505,106 | 12% |
| MetaSSR | `/api/bench` | 162,601 | 162,890 | 159,411 | 2% |
| Next.js | `/` | 824.88 | 828.55 | 820.80 | 1% |
| Next.js | `/api/bench` | 2,629.43 | 2,615.09 | 2,603.64 | 1% |

## Raw output

Raw wrk output is generated locally and is not committed. Re-run the benchmark to reproduce it.
