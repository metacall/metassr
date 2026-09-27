use std::error::Error;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod lockfree_side {
    use lockfree::map::Map;

    pub struct Bench(Map<u64, u64>);

    impl Bench {
        pub fn new() -> Self {
            Self(Map::new())
        }

        pub fn insert(&self, k: u64, v: u64) {
            self.0.insert(k, v);
        }

        pub fn get(&self, k: u64) -> bool {
            self.0.get(&k).is_some()
        }

        pub fn remove(&self, k: u64) {
            self.0.remove(&k);
        }
    }
}

mod lockfreedom_side {
    use lock_freedom::map::Map;

    pub struct Bench(Map<u64, u64>);

    impl Bench {
        pub fn new() -> Self {
            Self(Map::new())
        }

        pub fn insert(&self, k: u64, v: u64) {
            self.0.insert(k, v);
        }

        pub fn get(&self, k: u64) -> bool {
            self.0.get(&k).is_some()
        }

        pub fn remove(&self, k: u64) {
            self.0.remove(&k);
        }
    }
}

trait BenchMap: Send + Sync {
    fn new() -> Self;
    fn insert(&self, k: u64, v: u64);
    fn get(&self, k: u64) -> bool;
    fn remove(&self, k: u64);
}

impl BenchMap for lockfree_side::Bench {
    fn new() -> Self {
        Self::new()
    }

    fn insert(&self, k: u64, v: u64) {
        self.insert(k, v);
    }

    fn get(&self, k: u64) -> bool {
        self.get(k)
    }

    fn remove(&self, k: u64) {
        self.remove(k);
    }
}

impl BenchMap for lockfreedom_side::Bench {
    fn new() -> Self {
        Self::new()
    }

    fn insert(&self, k: u64, v: u64) {
        self.insert(k, v);
    }

    fn get(&self, k: u64) -> bool {
        self.get(k)
    }

    fn remove(&self, k: u64) {
        self.remove(k);
    }
}

fn run_insert<M: BenchMap>(threads: usize, ops: usize) -> Duration {
    let map = Arc::new(M::new());
    let start = Instant::now();
    thread::scope(|s| {
        for t in 0..threads {
            let map = Arc::clone(&map);
            s.spawn(move || {
                let mut acc = 0u64;
                for i in 0..ops {
                    let k = i as u64 * threads as u64 + t as u64;
                    map.insert(k, k + 1);
                    acc ^= k;
                }
                black_box(acc);
            });
        }
    });
    start.elapsed()
}

fn run_get<M: BenchMap>(threads: usize, ops: usize, keys: usize) -> Duration {
    let map = Arc::new(M::new());
    for k in 0..keys {
        map.insert(k as u64, k as u64 + 1);
    }
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..threads {
            let map = Arc::clone(&map);
            s.spawn(move || {
                let mut acc = 0u64;
                for i in 0..ops {
                    let k = (i % keys) as u64;
                    if map.get(k) {
                        acc ^= k;
                    }
                }
                black_box(acc);
            });
        }
    });
    start.elapsed()
}

fn run_mixed<M: BenchMap>(threads: usize, ops: usize, keys: usize) -> Duration {
    let map = Arc::new(M::new());
    for k in 0..keys {
        map.insert(k as u64, k as u64 + 1);
    }
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..threads {
            let map = Arc::clone(&map);
            s.spawn(move || {
                let mut acc = 0u64;
                for i in 0..ops {
                    let k = (i % keys) as u64;
                    if i % 100 == 0 {
                        map.remove(k);
                        map.insert(k, k + 1);
                    } else if map.get(k) {
                        acc ^= k;
                    }
                }
                black_box(acc);
            });
        }
    });
    start.elapsed()
}

struct Scenario {
    name: String,
    threads: usize,
    elapsed_secs: f64,
    ops: u64,
}

impl Scenario {
    fn ops_per_sec(&self) -> f64 {
        self.ops as f64 / self.elapsed_secs
    }
}

fn bench_one<M: BenchMap>(
    label: &str,
    name: &str,
    threads: usize,
    ops: usize,
    keys: usize,
) -> Scenario {
    let elapsed = match name {
        "Insert" => run_insert::<M>(threads, ops),
        "Get" => run_get::<M>(threads, ops, keys),
        "Mixed" => run_mixed::<M>(threads, ops, keys),
        _ => unreachable!(),
    };
    println!(
        "  {label:<12} {name:<6} threads={threads:<2} elapsed={:.3}s",
        elapsed.as_secs_f64()
    );
    Scenario {
        name: name.to_string(),
        threads,
        elapsed_secs: elapsed.as_secs_f64(),
        ops: (threads * ops) as u64,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let ops: usize = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(200_000);
    let keys: usize = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(10_000);
    let threads_list: Vec<usize> = args
        .get(3)
        .map(|v| v.split(',').filter_map(|t| t.parse().ok()).collect())
        .filter(|v: &Vec<usize>| !v.is_empty())
        .unwrap_or_else(|| vec![1, 2, 4, 8]);

    let scenarios = ["Insert", "Get", "Mixed"];
    println!("lock-map-bench: ops/thread={ops} keys={keys} threads={threads_list:?}");

    let mut results = Vec::new();
    for &threads in &threads_list {
        for &name in &scenarios {
            println!("[{name} threads={threads}]");
            let old = bench_one::<lockfree_side::Bench>("lockfree", name, threads, ops, keys);
            let new =
                bench_one::<lockfreedom_side::Bench>("lock_freedom", name, threads, ops, keys);
            results.push((old, new));
        }
    }

    println!();
    println!(
        "{:<6} {:<7} {:>16} {:>18} {:>10}",
        "Workload", "Threads", "lockfree ops/s", "lock_freedom ops/s", "Delta %"
    );
    for (old, new) in &results {
        let delta = (new.ops_per_sec() - old.ops_per_sec()) / old.ops_per_sec() * 100.0;
        println!(
            "{:<6} {:<7} {:>16.0} {:>18.0} {:>+9.2}%",
            old.name,
            old.threads,
            old.ops_per_sec(),
            new.ops_per_sec(),
            delta
        );
    }

    let stamp = now_stamp();
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("reports")
        .join("lock-freedom-vs-lockfree")
        .join(&stamp);
    fs::create_dir_all(&out_dir)?;

    let mut json = format!(
        "{{\n  \"timestamp\": \"{stamp}\",\n  \"ops_per_thread\": {ops},\n  \"keys\": {keys},\n  \"tests\": [\n"
    );
    for (old, new) in &results {
        json.push_str(&format!(
            "    {{\"name\": \"{}\", \"threads\": {}, \"lockfree_ops\": {:.0}, \"lock_freedom_ops\": {:.0}, \"lockfree_secs\": {:.4}, \"lock_freedom_secs\": {:.4}}},\n",
            old.name,
            old.threads,
            old.ops_per_sec(),
            new.ops_per_sec(),
            old.elapsed_secs,
            new.elapsed_secs,
        ));
    }
    json.truncate(json.len().saturating_sub(2));
    json.push_str("\n  ]\n}\n");
    fs::write(out_dir.join("results.json"), json)?;

    let mut md = String::new();
    md.push_str("# MetaSSR lock_freedom vs lockfree Benchmark\n\n");
    md.push_str(&format!("- Date: {}\n", stamp));
    md.push_str("- `lockfree` 0.5.1 (abandoned) vs `lock_freedom` 0.1.1 (maintained fork)\n");
    md.push_str(&format!(
        "- Ops per thread: {ops}, pre-populated keys: {keys}\n\n"
    ));
    md.push_str("| Workload | Threads | lockfree ops/s | lock_freedom ops/s | Delta |\n");
    md.push_str("|---|---:|---:|---:|---:|\n");
    for (old, new) in &results {
        let delta = (new.ops_per_sec() - old.ops_per_sec()) / old.ops_per_sec() * 100.0;
        md.push_str(&format!(
            "| {} | {} | {:.0} | {:.0} | {delta:+.2}% |\n",
            old.name,
            old.threads,
            old.ops_per_sec(),
            new.ops_per_sec(),
        ));
    }
    md.push_str("\nWorkloads (u64 keys):\n");
    md.push_str("- `Insert`: each thread inserts its own keys (no collisions).\n");
    md.push_str(
        "- `Get`: all threads read a shared key space (contended reads, the API hot path).\n",
    );
    md.push_str("- `Mixed`: reads with 1% remove+reinsert churn (hot-reload scenario).\n");
    md.push_str("\nPositive delta means `lock_freedom` is faster.\n");
    fs::write(out_dir.join("summary.md"), md)?;

    println!("\nReport written to {}", out_dir.display());
    Ok(())
}

fn now_stamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (d / 86400) as i64;
    let (y, m, day) = civil_from_days(days);
    let rem = d % 86400;
    format!(
        "{y}{:02}{:02}-{:02}{:02}{:02}",
        m,
        day,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
