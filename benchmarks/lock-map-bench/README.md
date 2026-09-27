# lock-map-bench

Micro-benchmark comparing the two lock-free `Map` implementations that
`metassr-api-handler` can use:

- [`lockfree`](https://gitlab.com/bzim/lockfree/) `0.5.1` — abandoned (no
  activity since ~2020).
- [`lock_freedom`](https://github.com/wyatt-herkamp/lock_freedom) `0.1.1` —
  maintained fork used by MetaSSR (see [#159](https://github.com/metacall/metassr/issues/159)).

This is a standalone crate (opt-out of the workspace) so the benchmark can
depend on both crates side-by-side without keeping `lockfree` in the main
`Cargo.lock`.

## Workloads

The three workloads mirror how `metassr-api-handler` uses the map (`u64` keys):

| Workload | What it does | Mirrors |
| --- | --- | --- |
| `Insert` | each thread inserts its own key range (no collisions) | `load_script` / `reload_script` |
| `Get` | all threads read a shared key space | `call_handler` (the hot HTTP path) |
| `Mixed` | reads with 1% remove + reinsert churn | hot reload under traffic |

## Usage

```sh
cargo run --release                    # 200k ops/thread, 10k keys, threads 1,2,4,8
cargo run --release -- 300000 30000    # custom ops/thread and key count
cargo run --release -- 300000 30000 1,2,4,8   # explicit thread list
```

Each run writes a timestamped report into
`../reports/lock-freedom-vs-lockfree/<timestamp>/` containing `results.json`
and `summary.md`.

## Caveats

Each scenario is a single timed run, so per-cell numbers are noisy — treat
the table as a rough comparison, not a precise regression benchmark. For
stable numbers, increase ops/thread and rerun a few times (or run on an idle
machine with pinned CPUs).