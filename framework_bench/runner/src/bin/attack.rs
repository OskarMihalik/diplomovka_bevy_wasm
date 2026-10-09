//! Goose attack for experiment A, run inside the Goose container by the host `runner`.
//!
//!   attack run                      configured by env: TARGET, ENDPOINT, USERS, RAMP_S, RUN_S, OUT
//!   attack probe <url> <timeout_s>  waits until <url> answers 200
//!
//! Goose drives the virtual users, the ramp-up and one HTTP client per user. Latency is recorded
//! here rather than taken from Goose metrics, for two reasons: Goose stores whole milliseconds
//! (rounded further above 100 ms), which cannot separate sub-millisecond endpoints, and it stops
//! the timer when the headers arrive, while this attack reads the full body. Goose metrics are
//! disabled so they add no load-generator overhead.

use goose::config::GooseConfiguration;
use goose::prelude::*;
use gumdrop::Options;
use serde::Serialize;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Log-linear histogram of microseconds: exact below 128 µs, then 128 sub-buckets per power of
/// two (at most 0.8 % relative error). Lock-free, shared by all users.
const SUB_BITS: u32 = 7;
const SUB: u64 = 1 << SUB_BITS;
const BUCKETS: usize = (SUB + (64 - SUB_BITS as u64) * SUB) as usize;

fn bucket_index(us: u64) -> usize {
    if us < SUB {
        return us as usize;
    }
    let exp = 63 - us.leading_zeros();
    let shift = exp - SUB_BITS;
    let sub = (us >> shift) - SUB;
    (SUB + shift as u64 * SUB + sub) as usize
}

/// Midpoint of the bucket in microseconds.
fn bucket_value(index: usize) -> f64 {
    let index = index as u64;
    if index < SUB {
        return index as f64;
    }
    let i = index - SUB;
    let shift = i / SUB;
    let lower = (SUB + i % SUB) << shift;
    lower as f64 + ((1u64 << shift) as f64 - 1.0) / 2.0
}

struct Recorder {
    buckets: Vec<AtomicU64>,
    sum_us: AtomicU64,
    max_us: AtomicU64,
    requests: AtomicU64,
    status_2xx: AtomicU64,
    status_other: AtomicU64,
    status_5xx: AtomicU64,
    transport_errors: AtomicU64,
    timeouts: AtomicU64,
    bytes: AtomicU64,
}

enum Outcome {
    Status(u16),
    Transport,
    Timeout,
}

impl Recorder {
    fn new() -> Self {
        Recorder {
            buckets: (0..BUCKETS).map(|_| AtomicU64::new(0)).collect(),
            sum_us: AtomicU64::new(0),
            max_us: AtomicU64::new(0),
            requests: AtomicU64::new(0),
            status_2xx: AtomicU64::new(0),
            status_other: AtomicU64::new(0),
            status_5xx: AtomicU64::new(0),
            transport_errors: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        }
    }

    fn record(&self, latency: Duration, outcome: Outcome, bytes: u64) {
        let us = latency.as_micros() as u64;
        self.buckets[bucket_index(us)].fetch_add(1, Ordering::Relaxed);
        self.sum_us.fetch_add(us, Ordering::Relaxed);
        self.max_us.fetch_max(us, Ordering::Relaxed);
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
        let counter = match outcome {
            Outcome::Status(s) if (200..300).contains(&s) => &self.status_2xx,
            Outcome::Status(s) if s >= 500 => &self.status_5xx,
            Outcome::Status(_) => &self.status_other,
            Outcome::Transport => &self.transport_errors,
            Outcome::Timeout => &self.timeouts,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Latency in ms at quantile q, from the bucket midpoints, capped at the recorded maximum
    /// (a midpoint can lie above the largest value in its bucket).
    fn quantile_ms(&self, counts: &[u64], total: u64, q: f64) -> f64 {
        if total == 0 {
            return 0.0;
        }
        let max_us = self.max_us.load(Ordering::Relaxed) as f64;
        let rank = ((q * total as f64).ceil() as u64).max(1);
        let mut cumulative = 0;
        for (index, count) in counts.iter().enumerate() {
            cumulative += count;
            if cumulative >= rank {
                return bucket_value(index).min(max_us) / 1000.0;
            }
        }
        max_us / 1000.0
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Endpoint {
    Plaintext,
    Json,
    EchoJson,
    JsonLarge,
    Cpu,
    Delay,
    File,
    Upload,
}

impl Endpoint {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "plaintext" => Endpoint::Plaintext,
            "json" => Endpoint::Json,
            "echo-json" => Endpoint::EchoJson,
            "json-large" => Endpoint::JsonLarge,
            "cpu" => Endpoint::Cpu,
            "delay" => Endpoint::Delay,
            "file" => Endpoint::File,
            "upload" => Endpoint::Upload,
            _ => return None,
        })
    }
}

struct Config {
    endpoint: Endpoint,
    /// Only requests that complete inside [window_start, window_end) after `t0` are recorded.
    t0: Instant,
    window_start: Duration,
    window_end: Duration,
    echo_body: Vec<u8>,
    upload_body: Vec<u8>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
static RECORDER: OnceLock<Recorder> = OnceLock::new();

/// Same ~2 KB payload as scripts/smoke.sh.
fn echo_body() -> Vec<u8> {
    let items: Vec<serde_json::Value> = (0..40)
        .map(|i| {
            serde_json::json!({
                "name": format!("item-{i:02}-north-facade"),
                "value": i as f64 * 1.5 + 0.25,
            })
        })
        .collect();
    serde_json::to_vec(&serde_json::json!({ "id": 42, "title": "Window frame review", "items": items }))
        .expect("echo body serializes")
}

/// 1 MiB of deterministic pseudo-random bytes (xorshift64).
fn upload_body() -> Vec<u8> {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    (0..1024 * 1024)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

async fn hit(user: &mut GooseUser) -> TransactionResult {
    let config = CONFIG.get().expect("config set before the attack starts");
    let builder = match config.endpoint {
        Endpoint::Plaintext => user.get_request_builder(&GooseMethod::Get, "plaintext")?,
        Endpoint::Json => user.get_request_builder(&GooseMethod::Get, "json")?,
        Endpoint::EchoJson => user
            .get_request_builder(&GooseMethod::Post, "echo-json")?
            .header("content-type", "application/json")
            .body(config.echo_body.clone()),
        Endpoint::JsonLarge => user.get_request_builder(&GooseMethod::Get, "json-large")?,
        Endpoint::Cpu => user.get_request_builder(&GooseMethod::Get, "cpu")?,
        Endpoint::Delay => user.get_request_builder(&GooseMethod::Get, "delay?ms=20")?,
        Endpoint::File => user.get_request_builder(&GooseMethod::Get, "file")?,
        Endpoint::Upload => user
            .get_request_builder(&GooseMethod::Post, "upload")?
            .header("content-type", "application/octet-stream")
            .body(config.upload_body.clone()),
    };
    let request = GooseRequest::builder().set_request_builder(builder).build();

    let started = Instant::now();
    let goose_response = user.request(request).await?;
    let mut bytes = 0u64;
    let outcome = match goose_response.response {
        Ok(mut response) => {
            let status = response.status().as_u16();
            // Read the whole body in chunks, so the latency covers the full response
            // without buffering multi-megabyte bodies per user.
            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) => bytes += chunk.len() as u64,
                    Ok(None) => break Outcome::Status(status),
                    Err(e) if e.is_timeout() => break Outcome::Timeout,
                    Err(_) => break Outcome::Transport,
                }
            }
        }
        Err(e) if e.is_timeout() => Outcome::Timeout,
        Err(_) => Outcome::Transport,
    };
    let finished = Instant::now();

    let since_start = finished.duration_since(config.t0);
    if since_start >= config.window_start && since_start < config.window_end {
        RECORDER
            .get()
            .expect("recorder set before the attack starts")
            .record(finished - started, outcome, bytes);
    }
    Ok(())
}

#[derive(Serialize)]
struct Summary {
    endpoint: String,
    target: String,
    users: usize,
    ramp_s: u64,
    run_s: u64,
    window_start_unix_ms: u64,
    window_end_unix_ms: u64,
    goose_maximum_users: usize,
    requests: u64,
    status_2xx: u64,
    status_other: u64,
    status_5xx: u64,
    transport_errors: u64,
    timeouts: u64,
    bytes: u64,
    rps: f64,
    mb_per_s: f64,
    lat_mean_ms: f64,
    lat_p50_ms: f64,
    lat_p90_ms: f64,
    lat_p95_ms: f64,
    lat_p99_ms: f64,
    lat_p999_ms: f64,
    lat_max_ms: f64,
    /// Non-empty buckets as [midpoint_us, count].
    histogram: Vec<(f64, u64)>,
}

fn env_or<T: std::str::FromStr>(name: &str, default: T) -> T {
    match std::env::var(name) {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| panic!("invalid value for {name}: {value}")),
        Err(_) => default,
    }
}

fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).expect("clock after 1970").as_millis() as u64
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let target: String = env_or("TARGET", "http://fw-server:8080/".to_string());
    let endpoint_name: String = env_or("ENDPOINT", "plaintext".to_string());
    let users: usize = env_or("USERS", 256);
    let ramp_s: u64 = env_or("RAMP_S", 10);
    let run_s: u64 = env_or("RUN_S", 60);
    let out: String = env_or("OUT", "/out/attack.json".to_string());
    let endpoint = Endpoint::parse(&endpoint_name)
        .ok_or_else(|| format!("unknown ENDPOINT {endpoint_name}"))?;

    // The window starts 1 s after the ramp and Goose keeps the users running 1 s past its end,
    // so the window contains only full load.
    let window_start = Duration::from_secs(ramp_s + 1);
    let window_end = window_start + Duration::from_secs(run_s);
    let t0 = Instant::now();
    let t0_unix = SystemTime::now();
    CONFIG
        .set(Config {
            endpoint,
            t0,
            window_start,
            window_end,
            echo_body: echo_body(),
            upload_body: upload_body(),
        })
        .map_err(|_| "config already set")?;
    RECORDER.set(Recorder::new()).map_err(|_| "recorder already set")?;

    let configuration = GooseConfiguration::parse_args_default(&[] as &[&str])?;
    let metrics = GooseAttack::initialize_with_config(configuration)?
        .register_scenario(scenario!("endpoint").register_transaction(transaction!(hit)))
        .set_default(GooseDefault::Host, target.as_str())?
        .set_default(GooseDefault::Users, users)?
        .set_default(GooseDefault::IncreaseTime, ramp_s as usize)?
        .set_default(GooseDefault::RunTime, (run_s + 2) as usize)?
        .set_default(GooseDefault::NoMetrics, true)?
        .set_default(GooseDefault::NoPrintMetrics, true)?
        .set_default(GooseDefault::NoTelnet, true)?
        .set_default(GooseDefault::NoWebSocket, true)?
        .set_default(GooseDefault::NoGzip, true)?
        .set_default(GooseDefault::Quiet, 1)?
        .execute()
        .await?;

    let recorder = RECORDER.get().expect("recorder set");
    let counts: Vec<u64> = recorder.buckets.iter().map(|b| b.load(Ordering::Relaxed)).collect();
    let requests = recorder.requests.load(Ordering::Relaxed);
    let bytes = recorder.bytes.load(Ordering::Relaxed);
    let seconds = run_s as f64;
    let summary = Summary {
        endpoint: endpoint_name,
        target,
        users,
        ramp_s,
        run_s,
        window_start_unix_ms: unix_ms(t0_unix + window_start),
        window_end_unix_ms: unix_ms(t0_unix + window_end),
        goose_maximum_users: metrics.maximum_users,
        requests,
        status_2xx: recorder.status_2xx.load(Ordering::Relaxed),
        status_other: recorder.status_other.load(Ordering::Relaxed),
        status_5xx: recorder.status_5xx.load(Ordering::Relaxed),
        transport_errors: recorder.transport_errors.load(Ordering::Relaxed),
        timeouts: recorder.timeouts.load(Ordering::Relaxed),
        bytes,
        rps: requests as f64 / seconds,
        mb_per_s: bytes as f64 / seconds / (1024.0 * 1024.0),
        lat_mean_ms: if requests == 0 {
            0.0
        } else {
            recorder.sum_us.load(Ordering::Relaxed) as f64 / requests as f64 / 1000.0
        },
        lat_p50_ms: recorder.quantile_ms(&counts, requests, 0.50),
        lat_p90_ms: recorder.quantile_ms(&counts, requests, 0.90),
        lat_p95_ms: recorder.quantile_ms(&counts, requests, 0.95),
        lat_p99_ms: recorder.quantile_ms(&counts, requests, 0.99),
        lat_p999_ms: recorder.quantile_ms(&counts, requests, 0.999),
        lat_max_ms: recorder.max_us.load(Ordering::Relaxed) as f64 / 1000.0,
        histogram: counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(index, count)| (bucket_value(index), *count))
            .collect(),
    };
    std::fs::write(&out, serde_json::to_vec_pretty(&summary)?)?;
    println!(
        "{} users={} requests={} rps={:.1} p50={:.3}ms p99={:.3}ms errors={}",
        summary.endpoint,
        summary.goose_maximum_users,
        summary.requests,
        summary.rps,
        summary.lat_p50_ms,
        summary.lat_p99_ms,
        summary.status_other + summary.status_5xx + summary.transport_errors + summary.timeouts,
    );
    Ok(())
}

/// Minimal HTTP/1.1 GET, so the probe does not depend on the Goose client.
async fn probe_once(host_port: &str, path: &str) -> bool {
    let attempt = async {
        let mut stream = tokio::net::TcpStream::connect(host_port).await.ok()?;
        let request = format!("GET {path} HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.ok()?;
        let mut head = [0u8; 12];
        stream.read_exact(&mut head).await.ok()?;
        Some(head.starts_with(b"HTTP/1.1 200"))
    };
    matches!(tokio::time::timeout(Duration::from_secs(2), attempt).await, Ok(Some(true)))
}

async fn probe(url: &str, timeout_s: u64) -> Result<(), Box<dyn std::error::Error>> {
    let rest = url.strip_prefix("http://").ok_or("probe url must start with http://")?;
    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let deadline = Instant::now() + Duration::from_secs(timeout_s);
    while Instant::now() < deadline {
        if probe_once(host_port, path).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(format!("{url} not ready after {timeout_s} s").into())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("run") => run().await,
        Some("probe") if args.len() == 4 => match args[3].parse() {
            Ok(timeout_s) => probe(&args[2], timeout_s).await,
            Err(_) => Err("timeout_s must be an integer".into()),
        },
        _ => Err("usage: attack run | attack probe <url> <timeout_s>".into()),
    };
    if let Err(e) = result {
        eprintln!("attack: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buckets_are_exact_below_128_us_and_monotonic_above() {
        for us in 0..SUB {
            assert_eq!(bucket_value(bucket_index(us)), us as f64);
        }
        let mut previous = 0;
        for us in [128, 129, 255, 256, 257, 1_000, 65_535, 1_000_000, 60_000_000, u64::MAX] {
            let index = bucket_index(us);
            assert!(index < BUCKETS);
            assert!(index >= previous);
            previous = index;
            let value = bucket_value(index);
            assert!((value - us as f64).abs() / us as f64 <= 1.0 / SUB as f64, "{us} -> {value}");
        }
    }

    #[test]
    fn quantiles_follow_recorded_values() {
        let recorder = Recorder::new();
        for us in 1..=1000u64 {
            recorder.record(Duration::from_micros(us), Outcome::Status(200), 0);
        }
        let counts: Vec<u64> = recorder.buckets.iter().map(|b| b.load(Ordering::Relaxed)).collect();
        let p50 = recorder.quantile_ms(&counts, 1000, 0.5) * 1000.0;
        let p99 = recorder.quantile_ms(&counts, 1000, 0.99) * 1000.0;
        assert!((p50 - 500.0).abs() <= 4.0, "p50 {p50}");
        assert!((p99 - 990.0).abs() <= 8.0, "p99 {p99}");
    }

    #[test]
    fn quantiles_never_exceed_the_maximum() {
        let recorder = Recorder::new();
        // 29_264_211 µs falls in a 131 ms wide bucket whose midpoint is above it.
        recorder.record(Duration::from_micros(29_264_211), Outcome::Status(200), 0);
        let counts: Vec<u64> = recorder.buckets.iter().map(|b| b.load(Ordering::Relaxed)).collect();
        assert_eq!(recorder.quantile_ms(&counts, 1, 0.99), 29_264.211);
    }
}
