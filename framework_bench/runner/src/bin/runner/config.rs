use serde::Deserialize;
use std::collections::BTreeMap;

/// framework_bench/bench.toml
#[derive(Deserialize)]
pub struct Bench {
    pub network: String,
    pub server_port: u16,
    pub server_memory: String,
    pub goose_cpus: String,
    pub csv: String,
    pub pilot_csv: String,
    pub raw_dir: String,
    pub load: Load,
    pub pilot: Pilot,
    #[serde(rename = "profile")]
    pub profiles: Vec<Profile>,
    #[serde(rename = "config")]
    pub configs: Vec<ServerConfig>,
    #[serde(rename = "cell")]
    pub cells: Vec<Cell>,
}

#[derive(Deserialize, Clone)]
pub struct Load {
    pub users: usize,
    pub ramp_s: u64,
    pub run_s: u64,
    pub idle_s: u64,
    pub cooldown_s: u64,
    pub reps: u32,
    pub sample_ms: u64,
}

#[derive(Deserialize)]
pub struct Pilot {
    pub reps: u32,
    pub run_s: u64,
}

#[derive(Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub rtt_ms: f64,
    pub jitter_ms: f64,
    pub loss_pct: f64,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub id: String,
    pub framework: String,
    pub image: String,
    pub cores: u32,
    pub cpuset: String,
    pub env: BTreeMap<String, String>,
}

#[derive(Deserialize, Clone)]
pub struct Cell {
    pub endpoint: String,
    pub profile: String,
    /// Run the cell only for configurations with these core counts (all if absent).
    #[serde(default)]
    pub cores: Option<Vec<u32>>,
}

impl Cell {
    pub fn applies_to(&self, config: &ServerConfig) -> bool {
        self.cores.as_ref().is_none_or(|cores| cores.contains(&config.cores))
    }
}

impl Bench {
    pub fn load(path: &str) -> Result<Bench, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let bench: Bench = toml::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
        for cell in &bench.cells {
            bench.profile(&cell.profile)?;
        }
        for config in &bench.configs {
            let cpus = parse_cpuset(&config.cpuset)?;
            if cpus.len() as u32 != config.cores {
                return Err(format!("{}: cpuset {} has {} CPUs, cores = {}", config.id, config.cpuset, cpus.len(), config.cores));
            }
        }
        Ok(bench)
    }

    pub fn profile(&self, id: &str) -> Result<&Profile, String> {
        self.profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("unknown profile {id}"))
    }
}

/// "2,4,6,12" or "16-27" -> CPU numbers.
pub fn parse_cpuset(cpuset: &str) -> Result<Vec<u32>, String> {
    let mut cpus = Vec::new();
    for part in cpuset.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let parse = |s: &str| s.parse::<u32>().map_err(|_| format!("invalid cpuset {cpuset}"));
        match part.split_once('-') {
            Some((a, b)) => cpus.extend(parse(a)?..=parse(b)?),
            None => cpus.push(parse(part)?),
        }
    }
    Ok(cpus)
}

/// splitmix64, so the run order is reproducible from the recorded seed.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}
