//! Experiment B client, Cornucopia variant (PLAN.md, section 6). The Node client
//! (`../node/src/index.ts`) runs the same scenario through Kysely; keep the two in step.
//!
//!   qbench --rep <n> [--workers 10] [--warmup 100] [--iterations 1000] [--smoke]
//!
//! Each worker has its own user and project and repeats the scenario: user_by_email,
//! tags_by_project, insert_tag, project_tx (transaction + delete). Every step takes its own pool
//! connection, as one backend request does. The run prints one CSV row to stdout (run.sh adds the
//! host columns in front); with --smoke it prints the content checksum and the row count instead.

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use model::cornucopia::queries::{
    auth::select_user,
    tags::{delete_project, insert_project, insert_tag, select_tags},
    users::insert_project_user,
};
use std::time::Instant;
use tokio_postgres::NoTls;

type DbPool = Pool<PostgresConnectionManager<NoTls>>;
type BoxError = Box<dyn std::error::Error + Send + Sync>;

const POOL_SIZE: u32 = 10;

struct Args {
    rep: u32,
    workers: u32,
    warmup: u32,
    iterations: u32,
    smoke: bool,
}

fn parse_args() -> Args {
    let mut args = Args { rep: 0, workers: 10, warmup: 100, iterations: 1000, smoke: false };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().and_then(|v| v.parse().ok()).expect("missing or bad value");
        match flag.as_str() {
            "--rep" => args.rep = value(),
            "--workers" => args.workers = value(),
            "--warmup" => args.warmup = value(),
            "--iterations" => args.iterations = value(),
            "--smoke" => args.smoke = true,
            other => panic!("unknown argument {other}"),
        }
    }
    args
}

/// Latencies in nanoseconds of the whole scenario and of each step, plus the counters.
#[derive(Default)]
struct Samples {
    scenario: Vec<u64>,
    steps: [Vec<u64>; 4],
    errors: u64,
    rows: u64,
    checksum: u64,
}

impl Samples {
    fn merge(&mut self, other: Samples) {
        self.scenario.extend(other.scenario);
        for (all, mine) in self.steps.iter_mut().zip(other.steps) {
            all.extend(mine);
        }
        self.errors += other.errors;
        self.rows += other.rows;
        self.checksum = self.checksum.wrapping_add(other.checksum);
    }
}

struct Worker {
    user_id: i32,
    project_id: i32,
    email: String,
}

/// One scenario. Returns the step latencies, the rows read or written, and the content checksum
/// (the same formula as the Node client, so both must agree on a fresh DB).
async fn scenario(pool: &DbPool, w: &Worker) -> Result<([u64; 4], u64, u64), BoxError> {
    let mut lat = [0u64; 4];
    let mut checksum = 0u64;

    // 1 user_by_email (login lookup)
    let t = Instant::now();
    let user = {
        let conn = pool.get().await?;
        select_user().bind(&*conn, &w.email).one().await?
    };
    lat[0] = t.elapsed().as_nanos() as u64;
    checksum += user.id as u64 + user.email.len() as u64 + user.username.len() as u64;

    // 2 tags_by_project (LIMIT 100, as in the backend)
    let t = Instant::now();
    let tags = {
        let conn = pool.get().await?;
        select_tags().bind(&*conn, &w.project_id, &100, &0).all().await?
    };
    lat[1] = t.elapsed().as_nanos() as u64;
    for tag in &tags {
        checksum += tag.id as u64
            + tag.status_id.unwrap_or(0) as u64
            + tag.title.len() as u64
            + (tag.position_x * 4.0).round() as u64;
    }

    // 3 insert_tag
    let t = Instant::now();
    let inserted = {
        let conn = pool.get().await?;
        insert_tag()
            .bind(&*conn, &"qbench tag", &w.project_id, &1.0, &2.0, &3.0, &w.user_id)
            .await?
    };
    lat[2] = t.elapsed().as_nanos() as u64;

    // 4 project_tx: project + admin membership in one transaction (as insert_project_service),
    // then the delete as a separate request (cascades to ProjectUser)
    let t = Instant::now();
    let project_id = {
        let mut conn = pool.get().await?;
        let tx = conn.transaction().await?;
        let id = insert_project()
            .bind(&tx, &"qbench tx project", &"created and deleted by qbench", &w.user_id)
            .one()
            .await?;
        insert_project_user().bind(&tx, &id, &w.user_id, &true).await?;
        tx.commit().await?;
        id
    };
    let deleted = {
        let conn = pool.get().await?;
        delete_project().bind(&*conn, &project_id).await?
    };
    lat[3] = t.elapsed().as_nanos() as u64;

    // rows: 1 user + tags + 1 tag + (1 project id + 1 membership) + 1 deleted project
    let rows = 1 + tags.len() as u64 + inserted + 2 + deleted;
    Ok((lat, rows, checksum))
}

async fn run_worker(pool: DbPool, w: Worker, iterations: u32) -> Samples {
    let mut s = Samples::default();
    for _ in 0..iterations {
        let t = Instant::now();
        match scenario(&pool, &w).await {
            Ok((lat, rows, checksum)) => {
                s.scenario.push(t.elapsed().as_nanos() as u64);
                for (all, v) in s.steps.iter_mut().zip(lat) {
                    all.push(v);
                }
                s.rows += rows;
                s.checksum = s.checksum.wrapping_add(checksum);
            }
            Err(error) => {
                s.errors += 1;
                eprintln!("worker {}: {error}", w.user_id);
            }
        }
    }
    s
}

async fn phase(pool: &DbPool, workers: u32, iterations: u32) -> Samples {
    let handles: Vec<_> = (1..=workers as i32)
        .map(|id| {
            let w = Worker { user_id: id, project_id: id, email: format!("qbench{id}@bench.local") };
            tokio::spawn(run_worker(pool.clone(), w, iterations))
        })
        .collect();
    let mut all = Samples::default();
    for handle in handles {
        all.merge(handle.await.expect("worker panicked"));
    }
    all
}

/// User + system CPU time of the whole process (all threads), in seconds.
fn cpu_seconds() -> f64 {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    let tv = |t: libc::timeval| t.tv_sec as f64 + t.tv_usec as f64 / 1e6;
    tv(usage.ru_utime) + tv(usage.ru_stime)
}

/// Peak resident set size (VmHWM) in MiB.
fn rss_peak_mib() -> f64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    status
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<f64>().ok())
        .map_or(0.0, |kb| kb / 1024.0)
}

/// Nearest-rank percentile in ms of a sorted slice.
fn pct(sorted: &[u64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = ((q * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1] as f64 / 1e6
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = parse_args();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "host=qbench-pg user=postgres password=postgres dbname=qbench".into());
    let manager = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;
    let pool = Pool::builder().max_size(POOL_SIZE).build(manager).await?;

    // open all connections before the warm-up, so connection setup is never measured
    {
        let mut held = Vec::new();
        for _ in 0..POOL_SIZE {
            held.push(pool.get().await?);
        }
    }

    if args.warmup > 0 {
        phase(&pool, args.workers, args.warmup).await;
    }
    let cpu_start = cpu_seconds();
    let start = Instant::now();
    let mut s = phase(&pool, args.workers, args.iterations).await;
    let duration_s = start.elapsed().as_secs_f64();
    let cpu_s = cpu_seconds() - cpu_start;

    if args.smoke {
        println!("{} {}", s.checksum, s.rows);
        return Ok(());
    }

    let n = s.scenario.len();
    s.scenario.sort_unstable();
    for step in &mut s.steps {
        step.sort_unstable();
    }
    let mean = s.scenario.iter().sum::<u64>() as f64 / n.max(1) as f64 / 1e6;
    let mut row = vec![
        "cornucopia".to_string(),
        "rust".into(),
        "cornucopia+bb8".into(),
        args.rep.to_string(),
        args.workers.to_string(),
        POOL_SIZE.to_string(),
        n.to_string(),
        format!("{duration_s:.3}"),
        format!("{:.1}", n as f64 / duration_s),
        s.errors.to_string(),
        s.rows.to_string(),
        format!("{mean:.4}"),
        format!("{:.4}", pct(&s.scenario, 0.50)),
        format!("{:.4}", pct(&s.scenario, 0.95)),
        format!("{:.4}", pct(&s.scenario, 0.99)),
    ];
    for step in &s.steps {
        row.push(format!("{:.4}", pct(step, 0.50)));
        row.push(format!("{:.4}", pct(step, 0.99)));
    }
    row.push(format!("{cpu_s:.3}"));
    row.push(format!("{:.4}", cpu_s * 1000.0 / n.max(1) as f64));
    row.push(format!("{:.1}", rss_peak_mib()));
    println!("{}", row.join(","));
    Ok(())
}
