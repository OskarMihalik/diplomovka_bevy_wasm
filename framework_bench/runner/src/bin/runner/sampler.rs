//! Samples CPU and memory of the server and Goose containers from cgroup v2, plus the CPU
//! package temperature and the frequency of the server's CPUs, in a background thread.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("clock after 1970").as_millis() as u64
}

/// cgroup v2 directory of a container (systemd or cgroupfs driver).
pub fn cgroup_dir(container_id: &str) -> Result<PathBuf, String> {
    for dir in [
        format!("/sys/fs/cgroup/system.slice/docker-{container_id}.scope"),
        format!("/sys/fs/cgroup/docker/{container_id}"),
    ] {
        if Path::new(&dir).join("cpu.stat").exists() {
            return Ok(PathBuf::from(dir));
        }
    }
    Err(format!("cgroup of container {container_id} not found"))
}

#[derive(Clone, Copy, Default)]
pub struct CgroupSample {
    pub usage_usec: u64,
    pub current: u64,
    pub anon: u64,
    pub file: u64,
    pub inactive_file: u64,
}

impl CgroupSample {
    /// Same working-set definition as `docker stats`.
    pub fn working_set(&self) -> u64 {
        self.current.saturating_sub(self.inactive_file)
    }
}

#[derive(Clone, Copy)]
pub struct Sample {
    pub t_ms: u64,
    pub server: CgroupSample,
    pub goose: CgroupSample,
    pub temp_c: f64,
    pub freq_mhz: f64,
}

fn read_keyed(path: &Path, keys: &[&str]) -> Vec<u64> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    keys.iter()
        .map(|key| {
            text.lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix(' ')?.trim().parse().ok())
                .unwrap_or(0)
        })
        .collect()
}

fn read_u64(path: &Path) -> u64 {
    std::fs::read_to_string(path).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

pub fn read_cgroup(dir: &Path) -> CgroupSample {
    let usage = read_keyed(&dir.join("cpu.stat"), &["usage_usec"]);
    let mem = read_keyed(&dir.join("memory.stat"), &["anon", "file", "inactive_file"]);
    CgroupSample {
        usage_usec: usage[0],
        current: read_u64(&dir.join("memory.current")),
        anon: mem[0],
        file: mem[1],
        inactive_file: mem[2],
    }
}

pub fn read_memory_peak(dir: &Path) -> u64 {
    read_u64(&dir.join("memory.peak"))
}

/// Processes the kernel OOM-killed in the cgroup (memory.events).
pub fn read_oom_kills(dir: &Path) -> u64 {
    read_keyed(&dir.join("memory.events"), &["oom_kill"])[0]
}

/// The x86_pkg_temp thermal zone, if present.
pub fn package_temp_path() -> Option<PathBuf> {
    std::fs::read_dir("/sys/class/thermal").ok()?.flatten().find_map(|entry| {
        let kind = std::fs::read_to_string(entry.path().join("type")).ok()?;
        (kind.trim() == "x86_pkg_temp").then(|| entry.path().join("temp"))
    })
}

pub struct Sampler {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<Vec<Sample>>,
}

impl Sampler {
    pub fn start(server: PathBuf, goose: PathBuf, server_cpus: &[u32], interval_ms: u64) -> Sampler {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let temp_path = package_temp_path();
        let freq_paths: Vec<PathBuf> = server_cpus
            .iter()
            .map(|cpu| PathBuf::from(format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_cur_freq")))
            .collect();
        let handle = std::thread::spawn(move || {
            let mut samples = Vec::new();
            let interval = Duration::from_millis(interval_ms);
            let mut next = Instant::now();
            while !stop_thread.load(Ordering::Relaxed) {
                let freq_khz: u64 = freq_paths.iter().map(|p| read_u64(p)).sum();
                samples.push(Sample {
                    t_ms: now_ms(),
                    server: read_cgroup(&server),
                    goose: read_cgroup(&goose),
                    temp_c: temp_path.as_deref().map(read_u64).unwrap_or(0) as f64 / 1000.0,
                    freq_mhz: freq_khz as f64 / freq_paths.len().max(1) as f64 / 1000.0,
                });
                next += interval;
                if let Some(wait) = next.checked_duration_since(Instant::now()) {
                    std::thread::sleep(wait);
                }
            }
            samples
        });
        Sampler { stop, handle }
    }

    pub fn stop(self) -> Vec<Sample> {
        self.stop.store(true, Ordering::Relaxed);
        self.handle.join().expect("sampler thread panicked")
    }
}

/// Summary of the samples inside [from_ms, to_ms].
pub struct Window<'a> {
    samples: Vec<&'a Sample>,
}

impl<'a> Window<'a> {
    pub fn new(samples: &'a [Sample], from_ms: u64, to_ms: u64) -> Window<'a> {
        Window {
            samples: samples.iter().filter(|s| s.t_ms >= from_ms && s.t_ms <= to_ms).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.samples.len() < 2
    }

    /// Average cores used, from the first to the last sample.
    pub fn cpu_avg(&self, pick: fn(&Sample) -> CgroupSample) -> f64 {
        match (self.samples.first(), self.samples.last()) {
            (Some(a), Some(b)) if b.t_ms > a.t_ms => {
                let used = pick(b).usage_usec.saturating_sub(pick(a).usage_usec) as f64;
                used / ((b.t_ms - a.t_ms) as f64 * 1000.0)
            }
            _ => 0.0,
        }
    }

    /// Cores used in each sampling interval.
    pub fn cpu_series(&self, pick: fn(&Sample) -> CgroupSample) -> Vec<f64> {
        self.samples
            .windows(2)
            .filter(|w| w[1].t_ms > w[0].t_ms)
            .map(|w| {
                pick(w[1]).usage_usec.saturating_sub(pick(w[0]).usage_usec) as f64
                    / ((w[1].t_ms - w[0].t_ms) as f64 * 1000.0)
            })
            .collect()
    }

    pub fn values(&self, f: impl Fn(&Sample) -> f64) -> Vec<f64> {
        self.samples.iter().map(|s| f(s)).collect()
    }
}

pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
}

pub fn max(values: &[f64]) -> f64 {
    values.iter().cloned().fold(0.0, f64::max)
}

/// Nearest-rank percentile.
pub fn percentile(values: &[f64], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let rank = ((q * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}
