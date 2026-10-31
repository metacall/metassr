# MetaSSR vs Next.js Benchmark — 2026-10-31

**Load:** `wrk -t12 -c1000 -d30s --latency` (headline) plus the canonical
scenarios (t1c10 / t4c50 / t8c200 / t12c500, 5s each).

**Method:** both apps run in Docker with `--network host` (no `docker-proxy`,
no veth). One server at a time. Headline figures are the **median of 3
interleaved rounds** (metassr → nextjs → metassr → …) with a 10s cooldown
between rounds.

**CPU cap:** the host is a laptop that thermally throttles under sustained
12-thread / 1000-connection load. All CPUs were pinned to 2.5 GHz
(`performance` governor, `scaling_min_freq = scaling_max_freq = 2.5 GHz`)
for both frameworks. Absolute numbers are therefore lower than a max-boost
run, but the comparison is stable and identical for both sides.

| App | Image | Versions |
| --- | --- | --- |
| MetaSSR | `metassr-bench-metassr-source:6879584` | built from source at `6879584`, includes per-request SSR fix #220 |
| Next.js | `metassr-bench-nextjs` | `next@16.3.7` from npm, production build |

Apps: `benchmarks/apps/metassr-app` and `benchmarks/apps/nextjs-app`. Both
expose `/` (the same 20-item list, server-rendered on every request) and
`/api/bench` (identical JSON handler and payload).

Host: 13th Gen Intel(R) Core(TM) i7-13650HX, 20 cores, 15Gi RAM, Linux 6.18.53-1-lts. Load: 7.57 5.47 4.18 6/2903 272667 before, 5.72 6.39 5.37 2/2774 283089 after.

> **Why this supersedes the previous reports:** the `2026-09-28` run was
> measured through Docker's userland proxy and is invalid. The `2026-10-03`
> run fixed the networking but used npm `1.0.0-alpha.2`, which predates
> #220; its SSR row is boot-time cached HTML, not per-request SSR. This run
> builds from source at the PR merge commit and measures real per-request
> rendering on both sides.

## Headline — `/api/bench` (equivalent endpoint, median of 3)

| Metric | MetaSSR | Next.js | Gain |
| --- | --- | --- | --- |
| Requests/sec | 107,116.71 | 1,469.79 | **72.88x faster** |
| Average Latency | 10.15ms | 126.75ms | **12.49x lower** |
| Transfer/sec | 16.96MB | 457.89KB | **37.93x higher** |
| Total Requests | 3,216,110 | 44,239 | **72.70x more** |
| Max Latency | 71.29ms | 1.97s | **27.63x lower** |
| Socket Errors | 0 | 237 (237 timeouts) | **Zero errors** |

## Headline — `/` (SSR page, median of 3)

| Metric | MetaSSR | Next.js | Gain |
| --- | --- | --- | --- |
| Requests/sec | 41,499.66 | 500.86 | **82.86x faster** |
| Average Latency | 25.36ms | 495.70ms | **19.55x lower** |
| Transfer/sec | 120.16MB | 5.72MB | **21.01x higher** |
| Total Requests | 1,249,104 | 15,057 | **82.96x more** |
| Max Latency | 158.38ms | 1.86s | **11.74x lower** |
| Socket Errors | 0 | 234 (234 timeouts) | **Zero errors** |

Note: the two SSR responses differ in size — MetaSSR serves a lean 2.9KB HTML
document, Next.js serves 11.6KB of HTML plus its runtime/chunk references.
The API row is the closer apples-to-apples comparison. Next.js runs as its
default single production process; MetaSSR uses all available cores.

## Run-to-run stability (headline)

| Framework | Endpoint | Round 1 | Round 2 | Round 3 | Spread |
| --- | --- | --- | --- | --- | --- |
| MetaSSR | `/` | 42,664.01 | 41,499.66 | 40,899.07 | 4.3% |
| MetaSSR | `/api/bench` | 83,755.77 | 107,116.71 | 110,744.42 | 25.2% |
| Next.js | `/` | 447.31 | 502.28 | 500.86 | 11.0% |
| Next.js | `/api/bench` | 1,451.17 | 1,469.79 | 1,504.79 | 3.6% |

The first MetaSSR `/api/bench` round is a warm-up outlier (83.8k vs 107–111k in rounds 2–3); the median is round 2 and the spread is reported above rather than hidden.

## Canonical scenarios (single pass, 5s each)

| Scenario | Endpoint | MetaSSR RPS | Next.js RPS | Gain |
| --- | --- | --- | --- | --- |
| Light (t1 c10) | `/` | 36,486 | 351 | 104.00x |
| Light (t1 c10) | `/api/bench` | 101,313 | 1,275 | 79.43x |
| Medium (t4 c50) | `/` | 43,379 | 444 | 97.74x |
| Medium (t4 c50) | `/api/bench` | 181,928 | 1,602 | 113.53x |
| Heavy (t8 c200) | `/` | 43,445 | 470 | 92.50x |
| Heavy (t8 c200) | `/api/bench` | 144,794 | 1,545 | 93.75x |
| Stress (t12 c500) | `/` | 42,783 | 484 | 88.43x |
| Stress (t12 c500) | `/api/bench` | 146,718 | 1,344 | 109.18x |

MetaSSR served all runs with **0 socket errors**; Next.js logged timeouts at
Heavy and Stress (and at the 1000-connection headline).

## Raw output

Full wrk output for every run is under [`raw/`](./raw/):
`raw/<framework>/<round>/root.txt`, `raw/<framework>/<round>/api.txt`, and
`raw/<framework>/1/canon-<scenario>-<endpoint>.txt`. `run.log` records the
environment, image IDs and per-run commands; `cpu-freq.log` samples the CPU
frequency during the run.
