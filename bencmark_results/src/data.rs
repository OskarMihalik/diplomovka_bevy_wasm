//! Reading the benchmark CSVs and deciding which rows belong to which comparison.

use serde::Deserialize;
use std::path::Path;

/// Only rows measured with these settings are used, everything else was a test run
pub const RESOLUTION: &str = "2088x1550";
pub const MSAA: &str = "4";
pub const SHADOWS: &str = "on";

pub struct Model {
    /// `model_name` in the CSV, the glb file name
    pub file: &'static str,
    /// name in tables
    pub name: &'static str,
    /// short name for axis labels
    pub short: &'static str,
    /// camera distance in m the comparison uses, rows with another distance are skipped
    pub distance: f64,
    /// doesn't fit into the 4 GB of wasm32 memory in Bevy, see readme.md
    pub wasm_oom: bool,
}

/// In the order they appear in tables and figures
pub const MODELS: &[Model] = &[
    Model {
        file: "barling_wrecks",
        name: "Barling Wrecks",
        short: "Barling",
        distance: 5.22,
        wasm_oom: false,
    },
    Model {
        file: "full_gameready_city_buildings_iv_hongkong",
        name: "Hong Kong City Buildings",
        short: "Hong Kong",
        distance: 10.,
        wasm_oom: false,
    },
    Model {
        file: "cai_mep_port_vietnam",
        name: "Cai Mep Port",
        short: "Cai Mep",
        distance: 5.22,
        wasm_oom: false,
    },
    Model {
        file: "administrative_and_warehouse_complex",
        name: "Administrative and Warehouse Complex",
        short: "Warehouse",
        distance: 19000.,
        wasm_oom: false,
    },
    Model {
        file: "london_city",
        name: "London City",
        short: "London",
        distance: 19000.,
        wasm_oom: true,
    },
    Model {
        file: "national_airport_by_aditya",
        name: "National Airport",
        short: "Airport",
        distance: 20.,
        wasm_oom: true,
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Platform {
    BevyWasm,
    BabylonWasm,
    BevyNative,
    // dedicated GPU, a reference only, every other platform runs on the integrated one
    // BevyNativeRtx,
}

impl Platform {
    pub const ALL: [Platform; 3] = [
        Platform::BevyWasm,
        Platform::BabylonWasm,
        Platform::BevyNative,
        // Platform::BevyNativeRtx,
    ];
    // The ones compared on the same integrated GPU
    pub const COMPARED: [Platform; 3] = [
        Platform::BevyWasm,
        Platform::BabylonWasm,
        Platform::BevyNative,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Platform::BevyWasm => "Bevy (WASM)",
            Platform::BabylonWasm => "Babylon.js",
            Platform::BevyNative => "Bevy (native)",
            // Platform::BevyNativeRtx => "Bevy (native, RTX 4070)",
        }
    }

    /// column name in the pgfplots data files
    pub fn key(self) -> &'static str {
        match self {
            Platform::BevyWasm => "bevy_wasm",
            Platform::BabylonWasm => "babylon_wasm",
            Platform::BevyNative => "bevy_native",
            // Platform::BevyNativeRtx => "bevy_native_rtx",
        }
    }

    fn of(engine: &str, browser: &str, gpu: &str) -> Option<Self> {
        let rtx = gpu.contains("NVIDIA");
        match (engine, browser == "native", rtx) {
            ("bevy", false, false) => Some(Platform::BevyWasm),
            ("babylon", false, false) => Some(Platform::BabylonWasm),
            ("bevy", true, false) => Some(Platform::BevyNative),
            // ("bevy", true, true) => Some(Platform::BevyNativeRtx),
            _ => None,
        }
    }
}

/// A row of either CSV, the columns the other file doesn't have stay 0
#[derive(Deserialize)]
pub struct Row {
    engine: String,
    msaa: String,
    shadows: String,
    camera_distance: f64,
    gpu: String,
    browser: String,
    resolution: String,
    model_name: String,
    // rendering_fps.csv
    #[serde(default)]
    pub avg_fps: f64,
    #[serde(default)]
    pub low_1_fps: f64,
    #[serde(default)]
    pub frame_min_ms: f64,
    #[serde(default)]
    pub frame_p50_ms: f64,
    #[serde(default)]
    pub frame_p95_ms: f64,
    #[serde(default)]
    pub frame_p99_ms: f64,
    #[serde(default)]
    pub frame_max_ms: f64,
    #[serde(default)]
    pub mesh_instances: f64,
    #[serde(default)]
    pub vertices: f64,
    #[serde(default)]
    pub triangles: f64,
    #[serde(default)]
    pub materials: f64,
    #[serde(default)]
    pub texture_mib: f64,
    // rendering_load.csv
    #[serde(default)]
    pub download_parse_ms: f64,
    #[serde(default)]
    pub mesh_creation_ms: f64,
    #[serde(default)]
    pub first_frame_ms: f64,
    #[serde(default)]
    pub total_ms: f64,
}

/// A row that belongs to one of the comparisons
pub struct Measurement {
    pub model: usize,
    pub platform: Platform,
    pub row: Row,
}

pub fn read_fps(path: &Path) -> Result<Vec<Measurement>, String> {
    read(path, true)
}

/// Load time doesn't depend on the camera, except the first frame, so the distance isn't checked
pub fn read_load(path: &Path) -> Result<Vec<Measurement>, String> {
    read(path, false)
}

fn read(path: &Path, check_distance: bool) -> Result<Vec<Measurement>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut measurements = Vec::new();
    let mut skipped = 0;
    for (line, row) in reader.deserialize::<Row>().enumerate() {
        let row = match row {
            Ok(row) => row,
            Err(error) => {
                // +2: header and 1-based lines
                eprintln!("{} line {}: skipped, {error}", path.display(), line + 2);
                skipped += 1;
                continue;
            }
        };
        let c = &row;
        let model = MODELS.iter().position(|m| m.file == c.model_name);
        let platform = Platform::of(&c.engine, &c.browser, &c.gpu);
        let settings_match = c.resolution == RESOLUTION && c.msaa == MSAA && c.shadows == SHADOWS;
        match (model, platform) {
            (Some(model), Some(platform))
                if settings_match
                    && (!check_distance
                        || (c.camera_distance - MODELS[model].distance).abs() < 0.01) =>
            {
                measurements.push(Measurement {
                    model,
                    platform,
                    row,
                })
            }
            _ => skipped += 1,
        }
    }
    eprintln!(
        "{}: {} rows used, {skipped} skipped (other settings, distance or unknown model)",
        path.display(),
        measurements.len()
    );
    Ok(measurements)
}
