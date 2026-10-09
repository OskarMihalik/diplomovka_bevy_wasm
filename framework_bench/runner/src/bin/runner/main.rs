//! Orchestrates experiment A (see framework_bench/PLAN.md, section 4.6). Run from the repo root:
//!
//!   cargo run --release -p framework_bench_runner --bin runner -- [options]
//!
//!   --build          build the fwbench-axum, fwbench-express and fwbench-attack images first
//!   --pilot          1 repetition with 20 s windows, written to the pilot CSV
//!   --reps N         number of repetitions (default from bench.toml)
//!   --seed S         seed for the run order (default: current time); rerun with the same seed to resume
//!   --only a,b       only these configurations
//!   --cells e/p,..   only these cells, e.g. json/P0,file/P5
//!   --dry-run        print the run order and exit
//!   --allow-turbo    run a full batch even though turbo is on or the governor is not performance
//!
//! Every repetition runs all configurations in a shuffled order, and every configuration runs its
//! cells in a shuffled order. Each cell gets a cold server container. Rows already in the CSV
//! (same seed, rep, config, endpoint, profile) are skipped, so an interrupted batch can resume.

mod config;
mod docker;
mod netem;
mod sampler;

use config::{Bench, Cell, Profile, Rng, ServerConfig, parse_cpuset};
use docker::docker;
use sampler::{Sample, Sampler, Window, max, mean, now_ms, percentile};
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const BENCH_TOML: &str = "framework_bench/bench.toml";
const SERVER: &str = "fw-server";
const GOOSE: &str = "fw-goose";
const MIB: f64 = 1024.0 * 1024.0;

struct Args {
    build: bool,
    pilot: bool,
    reps: Option<u32>,
    seed: Option<u64>,
    only: Option<Vec<String>>,
    cells: Option<Vec<String>>,
    dry_run: bool,
    allow_turbo: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args { build: false, pilot: false, reps: None, seed: None, only: None, cells: None, dry_run: false, allow_turbo: false };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        let list = |v: String| v.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>();
        match arg.as_str() {
            "--build" => args.build = true,
            "--pilot" => args.pilot = true,
            "--dry-run" => args.dry_run = true,
            "--allow-turbo" => args.allow_turbo = true,
            "--reps" => args.reps = Some(value("--reps")?.parse().map_err(|_| "invalid --reps")?),
            "--seed" => args.seed = Some(value("--seed")?.parse().map_err(|_| "invalid --seed")?),
            "--only" => args.only = Some(list(value("--only")?)),
            "--cells" => args.cells = Some(list(value("--cells")?)),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(args)
}

/// Host details recorded in every row.
struct HostMeta {
    host: String,
    os: String,
    kernel: String,
    cpu_model: String,
    turbo: String,
    governor: String,
    docker_version: String,
}

fn read_trim(path: &str) -> String {
    std::fs::read_to_string(path).map(|s| s.trim().to_string()).unwrap_or_else(|_| "unknown".into())
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

impl HostMeta {
    fn collect(server_cpu: u32) -> HostMeta {
        let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        HostMeta {
            host: read_trim("/etc/hostname"),
            os: os_release
                .lines()
                .find_map(|l| l.strip_prefix("PRETTY_NAME="))
                .unwrap_or("unknown")
                .trim_matches('"')
                .to_string(),
            kernel: read_trim("/proc/sys/kernel/osrelease"),
            cpu_model: cpuinfo
                .lines()
                .find_map(|l| l.strip_prefix("model name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
                .unwrap_or_else(|| "unknown".into()),
            turbo: match read_trim("/sys/devices/system/cpu/intel_pstate/no_turbo").as_str() {
                "0" => "on".into(),
                "1" => "off".into(),
                other => other.into(),
            },
            governor: read_trim(&format!("/sys/devices/system/cpu/cpu{server_cpu}/cpufreq/scaling_governor")),
            docker_version: docker(&["version", "--format", "{{.Server.Version}}"])
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".into()),
        }
    }
}

/// The subset of the attack's JSON summary the CSV needs.
#[derive(Deserialize)]
struct AttackSummary {
    users: usize,
    window_start_unix_ms: u64,
    window_end_unix_ms: u64,
    goose_maximum_users: usize,
    requests: u64,
    status_2xx: u64,
    status_other: u64,
    status_5xx: u64,
    transport_errors: u64,
    timeouts: u64,
    rps: f64,
    mb_per_s: f64,
    lat_mean_ms: f64,
    lat_p50_ms: f64,
    lat_p90_ms: f64,
    lat_p95_ms: f64,
    lat_p99_ms: f64,
    lat_p999_ms: f64,
    lat_max_ms: f64,
}

struct Job {
    rep: u32,
    config: ServerConfig,
    cell: Cell,
}

struct Ctx<'a> {
    bench: &'a Bench,
    meta: HostMeta,
    seed: u64,
    run_s: u64,
    raw_dir: PathBuf,
    goose_cgroup: PathBuf,
    goose_cores: usize,
    user: String,
}

type Row = Vec<(&'static str, String)>;

fn f(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Splits one CSV line, honouring quoted fields.
fn split_csv(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => fields.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    fields.push(field);
    fields
}

fn resume_key(seed: &str, rep: &str, config: &str, endpoint: &str, profile: &str) -> String {
    format!("{seed}|{rep}|{config}|{endpoint}|{profile}")
}

/// Keys of the rows already in the CSV.
fn done_keys(csv: &Path) -> Result<HashSet<String>, String> {
    let Ok(text) = std::fs::read_to_string(csv) else { return Ok(HashSet::new()) };
    let mut lines = text.lines();
    let Some(header) = lines.next() else { return Ok(HashSet::new()) };
    let header = split_csv(header);
    let col = |name: &str| {
        header.iter().position(|h| h == name).ok_or_else(|| format!("{}: missing column {name}", csv.display()))
    };
    let (seed, rep, config, endpoint, profile) = (col("seed")?, col("rep")?, col("config")?, col("endpoint")?, col("profile")?);
    Ok(lines
        .map(split_csv)
        .filter(|r| r.len() == header.len())
        .map(|r| resume_key(&r[seed], &r[rep], &r[config], &r[endpoint], &r[profile]))
        .collect())
}

fn append_row(csv: &Path, row: &Row) -> Result<(), String> {
    let header: Vec<&str> = row.iter().map(|(k, _)| *k).collect();
    let exists = csv.exists() && std::fs::metadata(csv).map(|m| m.len() > 0).unwrap_or(false);
    if exists {
        let text = std::fs::read_to_string(csv).map_err(|e| e.to_string())?;
        let existing = text.lines().next().unwrap_or_default();
        if existing != header.join(",") {
            return Err(format!("{} has a different header; move it away before running", csv.display()));
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv)
        .map_err(|e| format!("{}: {e}", csv.display()))?;
    let mut text = String::new();
    if !exists {
        text.push_str(&header.join(","));
        text.push('\n');
    }
    text.push_str(&row.iter().map(|(_, v)| csv_field(v)).collect::<Vec<_>>().join(","));
    text.push('\n');
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())
}

/// TCP RetransSegs of the Goose container's network namespace.
fn tcp_retrans() -> u64 {
    let text = docker(&["exec", GOOSE, "cat", "/proc/net/snmp"]).unwrap_or_default();
    let tcp: Vec<&str> = text.lines().filter(|l| l.starts_with("Tcp:")).collect();
    if tcp.len() < 2 {
        return 0;
    }
    let names: Vec<&str> = tcp[0].split_whitespace().collect();
    let values: Vec<&str> = tcp[1].split_whitespace().collect();
    names
        .iter()
        .position(|n| *n == "RetransSegs")
        .and_then(|i| values.get(i)?.parse().ok())
        .unwrap_or(0)
}

/// Average RTT from 20 pings, with the raw ping output.
fn measure_rtt() -> (f64, String) {
    let output = Command::new("docker")
        .args(["exec", GOOSE, "ping", "-q", "-c", "20", "-i", "0.2", "-W", "2", SERVER])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    // "rtt min/avg/max/mdev = 0.041/0.062/0.090/0.012 ms"
    let avg = output
        .lines()
        .find_map(|l| l.split_once(" = "))
        .and_then(|(_, v)| v.split('/').nth(1)?.parse().ok())
        .unwrap_or(f64::NAN);
    (avg, output)
}

fn write_resources(path: &Path, samples: &[Sample], ready_ms: u64, idle_s: u64, summary: &AttackSummary) -> Result<(), String> {
    let mut text = String::from("t_ms,phase,server_cpu_cores,server_ws_mib,server_anon_mib,server_file_mib,goose_cpu_cores,temp_c,freq_mhz\n");
    for pair in samples.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let dt_us = (b.t_ms.saturating_sub(a.t_ms)).max(1) as f64 * 1000.0;
        let phase = if b.t_ms < ready_ms + idle_s * 1000 {
            "idle"
        } else if b.t_ms < summary.window_start_unix_ms {
            "warmup"
        } else if b.t_ms <= summary.window_end_unix_ms {
            "measure"
        } else {
            "cooldown"
        };
        let _ = writeln!(
            text,
            "{},{phase},{:.3},{:.2},{:.2},{:.2},{:.3},{:.1},{:.0}",
            b.t_ms,
            b.server.usage_usec.saturating_sub(a.server.usage_usec) as f64 / dt_us,
            b.server.working_set() as f64 / MIB,
            b.server.anon as f64 / MIB,
            b.server.file as f64 / MIB,
            b.goose.usage_usec.saturating_sub(a.goose.usage_usec) as f64 / dt_us,
            b.temp_c,
            b.freq_mhz,
        );
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

fn sleep_s(seconds: u64) {
    std::thread::sleep(Duration::from_secs(seconds));
}

fn start_server(ctx: &Ctx, config: &ServerConfig) -> Result<(), String> {
    docker::remove(SERVER);
    let mut args: Vec<String> = [
        "run", "-d", "--name", SERVER, "--network", &ctx.bench.network,
        "--cpuset-cpus", &config.cpuset, "--memory", &ctx.bench.server_memory, "--memory-swap", &ctx.bench.server_memory,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(["-e".into(), format!("PORT={}", ctx.bench.server_port)]);
    for (key, value) in &config.env {
        args.extend(["-e".into(), format!("{key}={value}")]);
    }
    args.push(config.image.clone());
    docker(&args.iter().map(String::as_str).collect::<Vec<_>>())?;
    let url = format!("http://{SERVER}:{}/plaintext", ctx.bench.server_port);
    if let Err(e) = docker(&["exec", GOOSE, "attack", "probe", &url, "30"]) {
        let logs = docker(&["logs", SERVER]).unwrap_or_default();
        return Err(format!("{e}\nserver logs:\n{logs}"));
    }
    Ok(())
}

fn run_cell(ctx: &Ctx, job: &Job, profile: &Profile) -> Result<Row, String> {
    let load = &ctx.bench.load;
    let config = &job.config;
    let run_id = format!("{}_r{}_{}_{}_{}", ctx.seed, job.rep, config.id, job.cell.endpoint, profile.id);
    let dir = ctx.raw_dir.join(&run_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let server_cpus = parse_cpuset(&config.cpuset)?;

    start_server(ctx, config)?;
    let ready_ms = now_ms();
    let server_id = docker::container_id(SERVER)?;
    let server_cgroup = sampler::cgroup_dir(&server_id)?;
    let image_id = docker(&["inspect", "-f", "{{.Image}}", SERVER])?.trim().trim_start_matches("sha256:").chars().take(12).collect::<String>();
    let sampler = Sampler::start(server_cgroup.clone(), ctx.goose_cgroup.clone(), &server_cpus, load.sample_ms);
    sleep_s(load.idle_s);

    netem::apply(SERVER, profile)?;
    netem::apply(GOOSE, profile)?;
    let (rtt_ms, ping) = measure_rtt();
    let retrans_before = tcp_retrans();

    let out = format!("/out/{run_id}/attack.json");
    let attack = Command::new("docker")
        .args(["exec", "-u", &ctx.user])
        .args(["-e", &format!("TARGET=http://{SERVER}:{}/", ctx.bench.server_port)])
        .args(["-e", &format!("ENDPOINT={}", job.cell.endpoint)])
        .args(["-e", &format!("USERS={}", load.users)])
        .args(["-e", &format!("RAMP_S={}", load.ramp_s)])
        .args(["-e", &format!("RUN_S={}", ctx.run_s)])
        .args(["-e", &format!("OUT={out}")])
        .args([GOOSE, "attack", "run"])
        .output()
        .map_err(|e| format!("docker exec attack: {e}"))?;
    let attack_log = format!(
        "{}{}",
        String::from_utf8_lossy(&attack.stdout),
        String::from_utf8_lossy(&attack.stderr)
    );
    let _ = std::fs::write(dir.join("attack.log"), &attack_log);
    let attack_end_ms = now_ms();
    sleep_s(load.cooldown_s);
    let samples = sampler.stop();

    let retrans = tcp_retrans().saturating_sub(retrans_before);
    let (tc_server, drops_server) = netem::stats(SERVER);
    let (tc_goose, drops_goose) = netem::stats(GOOSE);
    let mem_peak = sampler::read_memory_peak(&server_cgroup);
    // A server killed under load (e.g. by the memory limit) must not look like a normal row.
    let oom_kills = sampler::read_oom_kills(&server_cgroup);
    let server_state = docker(&["inspect", "-f", "{{.State.Status}}", SERVER])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    netem::clear(GOOSE);
    let _ = std::fs::write(dir.join("netem.txt"), format!("{ping}\n# server\n{tc_server}\n# goose\n{tc_goose}"));
    let _ = std::fs::write(dir.join("server.log"), docker(&["logs", SERVER]).unwrap_or_default());
    docker::remove(SERVER);

    if !attack.status.success() {
        return Err(format!("attack failed:\n{attack_log}"));
    }
    let summary: AttackSummary = serde_json::from_slice(
        &std::fs::read(dir.join("attack.json")).map_err(|e| format!("attack.json: {e}"))?,
    )
    .map_err(|e| format!("attack.json: {e}"))?;
    if summary.goose_maximum_users < summary.users {
        return Err(format!("Goose launched only {} of {} users", summary.goose_maximum_users, summary.users));
    }
    write_resources(&dir.join("resources.csv"), &samples, ready_ms, load.idle_s, &summary)?;

    let server = |s: &Sample| s.server;
    let goose = |s: &Sample| s.goose;
    let idle = Window::new(&samples, ready_ms, ready_ms + load.idle_s * 1000);
    let measure = Window::new(&samples, summary.window_start_unix_ms, summary.window_end_unix_ms);
    let cooldown = Window::new(&samples, attack_end_ms, u64::MAX);
    if measure.is_empty() {
        return Err("no resource samples inside the measurement window".into());
    }
    let ws = |s: &Sample| s.server.working_set() as f64 / MIB;
    let cpu_avg = measure.cpu_avg(server);
    let goose_cpu = measure.cpu_avg(goose);
    let window_s = (summary.window_end_unix_ms - summary.window_start_unix_ms) as f64 / 1000.0;
    let requests = summary.requests as f64;
    let errors = summary.requests - summary.status_2xx;

    Ok(vec![
        ("run_id", run_id),
        ("date", command_output("date", &["+%Y-%m-%d %H:%M:%S"])),
        ("host", ctx.meta.host.clone()),
        ("os", ctx.meta.os.clone()),
        ("kernel", ctx.meta.kernel.clone()),
        ("cpu_model", ctx.meta.cpu_model.clone()),
        ("turbo", ctx.meta.turbo.clone()),
        ("governor", ctx.meta.governor.clone()),
        ("docker_version", ctx.meta.docker_version.clone()),
        ("server_image", image_id),
        ("seed", ctx.seed.to_string()),
        ("rep", job.rep.to_string()),
        ("config", config.id.clone()),
        ("framework", config.framework.clone()),
        ("cores", config.cores.to_string()),
        ("cpuset", config.cpuset.clone()),
        ("profile", profile.id.clone()),
        ("rtt_ms", f(profile.rtt_ms, 1)),
        ("jitter_ms", f(profile.jitter_ms, 1)),
        ("loss_pct", f(profile.loss_pct, 2)),
        ("measured_rtt_ms", f(rtt_ms, 3)),
        ("netem_drops_client", drops_goose.to_string()),
        ("netem_drops_server", drops_server.to_string()),
        ("tcp_retrans", retrans.to_string()),
        ("client_saturated", (goose_cpu / ctx.goose_cores as f64 > 0.8).to_string()),
        ("goose_cpu_avg_cores", f(goose_cpu, 3)),
        ("cpu_temp_max_c", f(max(&measure.values(|s| s.temp_c)), 1)),
        ("cpu_freq_avg_mhz", f(mean(&measure.values(|s| s.freq_mhz)), 0)),
        ("server_state", server_state),
        ("oom_kills", oom_kills.to_string()),
        ("endpoint", job.cell.endpoint.clone()),
        ("users", summary.users.to_string()),
        ("duration_s", f(window_s, 1)),
        ("requests", summary.requests.to_string()),
        ("rps", f(summary.rps, 2)),
        ("mb_per_s", f(summary.mb_per_s, 2)),
        ("err_pct", f(if requests > 0.0 { errors as f64 / requests * 100.0 } else { 0.0 }, 4)),
        ("status_other", summary.status_other.to_string()),
        ("status_5xx", summary.status_5xx.to_string()),
        ("transport_errors", summary.transport_errors.to_string()),
        ("timeouts", summary.timeouts.to_string()),
        ("lat_mean", f(summary.lat_mean_ms, 3)),
        ("lat_p50", f(summary.lat_p50_ms, 3)),
        ("lat_p90", f(summary.lat_p90_ms, 3)),
        ("lat_p95", f(summary.lat_p95_ms, 3)),
        ("lat_p99", f(summary.lat_p99_ms, 3)),
        ("lat_p999", f(summary.lat_p999_ms, 3)),
        ("lat_max", f(summary.lat_max_ms, 3)),
        ("cpu_idle_cores", f(idle.cpu_avg(server), 4)),
        ("cpu_avg_cores", f(cpu_avg, 3)),
        ("cpu_p95_cores", f(percentile(&measure.cpu_series(server), 0.95), 3)),
        ("cpu_util_pct", f(cpu_avg / config.cores as f64 * 100.0, 1)),
        ("cpu_s_per_1k_req", f(if requests > 0.0 { cpu_avg * window_s / requests * 1000.0 } else { 0.0 }, 4)),
        ("mem_idle_mib", f(mean(&idle.values(ws)), 2)),
        ("mem_avg_mib", f(mean(&measure.values(ws)), 2)),
        ("mem_p95_mib", f(percentile(&measure.values(ws), 0.95), 2)),
        ("mem_max_mib", f(max(&measure.values(ws)), 2)),
        ("mem_peak_mib", f(mem_peak as f64 / MIB, 2)),
        ("mem_after_mib", f(cooldown.values(ws).last().copied().unwrap_or(0.0), 2)),
        ("mem_anon_peak_mib", f(max(&measure.values(|s| s.server.anon as f64 / MIB)), 2)),
    ])
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if !Path::new(BENCH_TOML).exists() {
        return Err(format!("{BENCH_TOML} not found; run from the repository root"));
    }
    let bench = Bench::load(BENCH_TOML)?;
    if args.build {
        docker::build_images()?;
    }
    for image in ["fwbench-axum", "fwbench-express", "fwbench-attack"] {
        docker(&["image", "inspect", image]).map_err(|_| format!("image {image} missing; run with --build"))?;
    }

    let seed = args.seed.unwrap_or_else(now_ms);
    let reps = args.reps.unwrap_or(if args.pilot { bench.pilot.reps } else { bench.load.reps });
    let run_s = if args.pilot { bench.pilot.run_s } else { bench.load.run_s };
    let csv = PathBuf::from(if args.pilot { &bench.pilot_csv } else { &bench.csv });
    let done = done_keys(&csv)?;

    let mut rng = Rng::new(seed);
    let mut jobs = Vec::new();
    for rep in 1..=reps {
        let mut configs = bench.configs.clone();
        rng.shuffle(&mut configs);
        for config in configs {
            let mut cells: Vec<Cell> = bench.cells.iter().filter(|c| c.applies_to(&config)).cloned().collect();
            rng.shuffle(&mut cells);
            for cell in cells {
                jobs.push(Job { rep, config: config.clone(), cell });
            }
        }
    }
    let total_planned = jobs.len();
    jobs.retain(|job| {
        args.only.as_ref().is_none_or(|only| only.contains(&job.config.id))
            && args.cells.as_ref().is_none_or(|cells| cells.contains(&format!("{}/{}", job.cell.endpoint, job.cell.profile)))
            && !done.contains(&resume_key(&seed.to_string(), &job.rep.to_string(), &job.config.id, &job.cell.endpoint, &job.cell.profile))
    });

    let load = &bench.load;
    // Container start, probe, netem and ping take about 10 s per cell on top of the phases.
    let cell_s = 10 + load.idle_s + load.ramp_s + run_s + 3 + load.cooldown_s;
    println!(
        "seed {seed}: {} of {total_planned} cells to run, about {:.1} h, writing {}",
        jobs.len(),
        (jobs.len() as u64 * cell_s) as f64 / 3600.0,
        csv.display()
    );
    if args.dry_run {
        for job in &jobs {
            println!("rep {} {:<11} {}/{}", job.rep, job.config.id, job.cell.endpoint, job.cell.profile);
        }
        return Ok(());
    }
    if jobs.is_empty() {
        return Ok(());
    }

    let raw_dir = PathBuf::from(&bench.raw_dir);
    std::fs::create_dir_all(&raw_dir).map_err(|e| format!("{}: {e}", raw_dir.display()))?;
    let raw_dir = raw_dir.canonicalize().map_err(|e| e.to_string())?;
    docker::ensure_network(&bench.network)?;
    docker::remove(SERVER);
    docker::remove(GOOSE);
    let mount = format!("{}:/out", raw_dir.display());
    docker(&[
        "run", "-d", "--name", GOOSE, "--network", &bench.network,
        "--cpuset-cpus", &bench.goose_cpus, "-v", &mount, "fwbench-attack", "sleep", "infinity",
    ])?;
    let goose_cgroup = sampler::cgroup_dir(&docker::container_id(GOOSE)?)?;

    let first_server_cpu = parse_cpuset(&bench.configs[0].cpuset)?[0];
    let ctx = Ctx {
        bench: &bench,
        meta: HostMeta::collect(first_server_cpu),
        seed,
        run_s,
        raw_dir,
        goose_cgroup,
        goose_cores: parse_cpuset(&bench.goose_cpus)?.len(),
        user: format!("{}:{}", command_output("id", &["-u"]), command_output("id", &["-g"])),
    };
    // Turbo makes the laptop throttle at 100 °C, so a full batch must run with it off.
    if ctx.meta.turbo != "off" || ctx.meta.governor != "performance" {
        let message = format!(
            "turbo is {} and the governor is {}; run `sudo framework_bench/scripts/host_prepare.sh` first",
            ctx.meta.turbo, ctx.meta.governor
        );
        if args.pilot || args.allow_turbo {
            println!("warning: {message}");
        } else {
            docker::remove(GOOSE);
            return Err(format!("{message} (or pass --allow-turbo)"));
        }
    }

    let mut failures = 0;
    for (i, job) in jobs.iter().enumerate() {
        let profile = bench.profile(&job.cell.profile)?;
        print!("[{}/{}] rep {} {:<11} {:<10} {} ... ", i + 1, jobs.len(), job.rep, job.config.id, job.cell.endpoint, profile.id);
        let _ = std::io::stdout().flush();
        match run_cell(&ctx, job, profile).and_then(|row| {
            let mut summary = format!(
                "rps {} p50 {} ms p99 {} ms err {}% cpu {} cores mem {} MiB",
                row_value(&row, "rps"), row_value(&row, "lat_p50"), row_value(&row, "lat_p99"),
                row_value(&row, "err_pct"), row_value(&row, "cpu_avg_cores"), row_value(&row, "mem_max_mib")
            );
            if row_value(&row, "server_state") != "running" || row_value(&row, "oom_kills") != "0" {
                summary.push_str(&format!(
                    " WARNING: server {}, {} OOM kill(s)",
                    row_value(&row, "server_state"), row_value(&row, "oom_kills")
                ));
            }
            if row_value(&row, "client_saturated") == "true" {
                summary.push_str(" WARNING: Goose saturated");
            }
            append_row(&csv, &row).map(|_| summary)
        }) {
            Ok(summary) => println!("{summary}"),
            Err(e) => {
                failures += 1;
                docker::remove(SERVER);
                netem::clear(GOOSE);
                println!("FAILED\n{e}");
            }
        }
    }
    docker::remove(GOOSE);
    if failures > 0 {
        return Err(format!("{failures} cell(s) failed; rerun with --seed {seed} to retry them"));
    }
    println!("done, seed {seed}");
    Ok(())
}

fn row_value<'a>(row: &'a Row, key: &str) -> &'a str {
    row.iter().find(|(k, _)| *k == key).map(|(_, v)| v.as_str()).unwrap_or("?")
}

fn main() {
    if let Err(e) = run() {
        eprintln!("runner: {e}");
        std::process::exit(1);
    }
}
