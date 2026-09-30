//! Benchmark settings, from command line arguments natively and from the url query on the web.
//!
//! Native: `bevy_rendering_benchmark --model ../backend/assets/models/14.glb --gpu low`
//! Web:    `http://localhost:8082/?model=http://localhost:8090/14.glb&gpu=low`
//!
//! Options (the same names are used as query parameters):
//! - `model`      path to a .glb file, or an http(s) url (required)
//! - `id`         model id for the CSV, defaults to the file name if it is a number (`14.glb`)
//! - `name`       model name for the CSV, defaults to the name of a known id, else the file name
//! - `version`    model version for the CSV, defaults to 1
//! - `runs`       how many measured runs, default 5
//! - `duration`   seconds of one run, the camera does one full orbit in this time, default 60
//! - `warmup`     seconds before the first run that are not measured, default 5
//! - `gpu`        `low` or `high`, the wgpu power preference (default from `WGPU_POWER_PREF`, else high)
//! - `resolution` window size in physical pixels, `1920x1080` by default
//! - `msaa`       1, 2, 4 or 8 samples, default 4 like bevy_diplomovka
//! - `shadows`    `on` or `off`, shadows of the point light, default on like bevy_diplomovka
//! - `distance`   meters from the camera to the center of the orbit, default 5.22 like
//!                bevy_diplomovka, the model is scaled to 0.25 so a 100 m model is 25 m wide
//! - `out`        native only, directory to append `rendering_fps.csv` and `rendering_load.csv` to
//! - `exit`       native only, `off` keeps the window open after the benchmark

use bevy::prelude::*;
use bevy::render::settings::PowerPreference;

/// Known models, so the CSV rows can be joined with bencmark_results/fps.csv by id
const MODELS: &[(u32, &str, u32)] = &[
    (12, "Barling Wrecks", 1),
    (13, "King's hall", 1),
    (14, "Cathedral", 1),
    (15, "Hong Kong City Buiildings", 1),
    (16, "Acrisure Stadium", 1),
    (17, "Cei Mep Port", 1),
    (18, "Administrative and Warehouse Comples", 1),
];

#[derive(Resource, Clone, Debug)]
pub struct BenchConfig {
    /// file path or url as given
    pub model: String,
    pub model_id: String,
    pub model_name: String,
    pub model_version: String,
    pub runs: usize,
    pub duration: f32,
    pub warmup: f32,
    pub power_preference: Option<PowerPreference>,
    pub width: u32,
    pub height: u32,
    pub msaa: u32,
    pub shadows: bool,
    /// orbit radius in meters
    pub distance: f32,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub out_dir: Option<String>,
    pub exit_when_done: bool,
}

impl BenchConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::parse(&args()?)
    }

    fn parse(args: &[(String, String)]) -> Result<Self, String> {
        let get = |key: &str| {
            args.iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        let number = |key: &str, default: f32| -> Result<f32, String> {
            get(key).map_or(Ok(default), |v| {
                v.parse()
                    .map_err(|_| format!("`{key}` must be a number, got `{v}`"))
            })
        };

        let model = get("model")
            .ok_or("missing `model`, a path to a .glb file or an http(s) url")?
            .to_string();
        let stem = model
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or_default()
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .trim_end_matches(".glb")
            .trim_end_matches(".gltf")
            .to_string();
        let model_id = get("id").map(str::to_string).unwrap_or_else(|| {
            if stem.parse::<u32>().is_ok() {
                stem.clone()
            } else {
                String::new()
            }
        });
        let known = MODELS.iter().find(|(id, _, _)| id.to_string() == model_id);
        let model_name = get("name")
            .map(str::to_string)
            .or_else(|| known.map(|(_, name, _)| name.to_string()))
            .unwrap_or_else(|| stem.clone());
        let model_version = get("version")
            .map(str::to_string)
            .or_else(|| known.map(|(_, _, version)| version.to_string()))
            .unwrap_or_else(|| "1".to_string());

        let power_preference = match get("gpu") {
            None => None,
            Some("low") => Some(PowerPreference::LowPower),
            Some("high") => Some(PowerPreference::HighPerformance),
            Some(other) => return Err(format!("`gpu` must be `low` or `high`, got `{other}`")),
        };
        let (width, height) = match get("resolution") {
            None => (2088, 1550),
            Some(resolution) => resolution
                .split_once('x')
                .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                .ok_or(format!(
                    "`resolution` must look like 1920x1080, got `{resolution}`"
                ))?,
        };
        let msaa = number("msaa", 4.)? as u32;
        if ![1, 2, 4, 8].contains(&msaa) {
            return Err(format!("`msaa` must be 1, 2, 4 or 8, got {msaa}"));
        }
        let on_off = |key: &str, default: bool| match get(key) {
            None => Ok(default),
            Some("on" | "true" | "1") => Ok(true),
            Some("off" | "false" | "0") => Ok(false),
            Some(other) => Err(format!("`{key}` must be `on` or `off`, got `{other}`")),
        };

        Ok(Self {
            model,
            model_id,
            model_name,
            model_version,
            runs: (number("runs", 5.)? as usize).max(1),
            duration: number("duration", 60.)?.max(1.),
            warmup: number("warmup", 5.)?.max(0.),
            power_preference,
            width,
            height,
            msaa,
            shadows: on_off("shadows", true)?,
            distance: number("distance", crate::scene::DEFAULT_DISTANCE)?.max(0.01),
            out_dir: get("out").map(str::to_string),
            exit_when_done: on_off("exit", true)?,
        })
    }

    pub fn is_url(&self) -> bool {
        self.model.starts_with("http://") || self.model.starts_with("https://")
    }

    pub fn msaa(&self) -> Msaa {
        match self.msaa {
            1 => Msaa::Off,
            2 => Msaa::Sample2,
            8 => Msaa::Sample8,
            _ => Msaa::Sample4,
        }
    }
}

/// `--key value` pairs
#[cfg(not(target_arch = "wasm32"))]
fn args() -> Result<Vec<(String, String)>, String> {
    let mut args = std::env::args().skip(1);
    let mut pairs = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            return Err(USAGE.to_string());
        }
        let Some(key) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument `{arg}`\n{USAGE}"));
        };
        // `--key=value` or `--key value`
        let (key, value) = match key.split_once('=') {
            Some((key, value)) => (key.to_string(), value.to_string()),
            None => (
                key.to_string(),
                args.next().ok_or(format!("missing value for `--{key}`"))?,
            ),
        };
        pairs.push((key, value));
    }
    Ok(pairs)
}

#[cfg(not(target_arch = "wasm32"))]
const USAGE: &str = "usage: bevy_rendering_benchmark --model <path.glb | url> [--id N] [--name NAME] \
[--version N] [--runs 5] [--duration 60] [--warmup 5] [--gpu low|high] [--resolution 1920x1080] \
[--msaa 4] [--shadows on|off] [--distance 5.22] [--out DIR] [--exit on|off]";

/// `?key=value&...` from the page url, a relative `model` is resolved against the page
#[cfg(target_arch = "wasm32")]
fn args() -> Result<Vec<(String, String)>, String> {
    let location = web_sys::window().ok_or("no window")?.location();
    let href = location.href().map_err(|_| "no page url")?;
    let search = location.search().unwrap_or_default();
    let mut pairs = Vec::new();
    for pair in search
        .trim_start_matches('?')
        .split('&')
        .filter(|p| !p.is_empty())
    {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let decode = |s: &str| {
            js_sys::decode_uri_component(&s.replace('+', " "))
                .map(String::from)
                .unwrap_or_else(|_| s.to_string())
        };
        let (key, mut value) = (decode(key), decode(value));
        if key == "model" {
            if let Ok(url) = web_sys::Url::new_with_base(&value, &href) {
                value = url.href();
            }
        }
        pairs.push((key, value));
    }
    Ok(pairs)
}
