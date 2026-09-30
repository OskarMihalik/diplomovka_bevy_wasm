//! The benchmark itself, ported from bevy_diplomovka's debug_tools.rs (F9). It starts on its own
//! once the model is shown: a warmup that isn't measured, then `runs` runs of `duration` seconds
//! each, in which the camera does one full orbit. Every run prints one CSV row, the same columns
//! as bencmark_results/fps.csv with `engine,msaa,shadows,camera_distance` in front.

use bevy::app::AppExit;
use bevy::ecs::system::SystemParam;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::window::PrimaryWindow;
use std::f32::consts::TAU;

use crate::config::BenchConfig;
use crate::load_timing::ModelLoadTiming;
use crate::measure::MeasureModel;
use crate::scene::{BenchCamera, BenchModel, orbit_transform};
use crate::system_info;

/// Columns in front of every CSV row, fps.csv and load.csv start the same without the first three
const COMMON_CSV_HEADER: &str = "engine,msaa,shadows,camera_distance,date,pc,os,cpu_cores,gpu,graphics_backend,browser,browser_version,build,resolution,model_id,model_name,model_version";
const FPS_CSV_HEADER: &str = "duration_s,frames,avg_fps,median_fps,low_1_fps,low_01_fps,frame_avg_ms,frame_min_ms,frame_p50_ms,frame_p95_ms,frame_p99_ms,frame_max_ms,frame_stddev_ms,mesh_instances,unique_meshes,vertices,triangles,materials,textures,texture_mib,entities";

pub struct BenchPlugin;

impl Plugin for BenchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bench>().add_systems(
            Update,
            (
                start_warmup,
                record_frame_time,
                orbit_camera,
                finish_run,
            )
                .chain(),
        );
    }
}

#[derive(Default)]
enum Phase {
    #[default]
    Loading,
    /// not measured, gives pipelines and shaders time to compile
    Warmup { elapsed: f32 },
    Running(BenchmarkRun),
    Done,
}

#[derive(Resource, Default)]
struct Bench {
    phase: Phase,
    /// fps rows of the finished runs, printed together at the end for copying
    rows: Vec<String>,
}

struct BenchmarkRun {
    /// 1-based index of this run out of `runs`
    index: usize,
    elapsed: f32,
    frames: Vec<f32>,
    /// sanity check independent of the per-frame deltas: every update, including zero-length ones
    updates: usize,
    started: Instant,
}

impl BenchmarkRun {
    fn new(index: usize) -> Self {
        Self {
            index,
            elapsed: 0.,
            frames: Vec::new(),
            updates: 0,
            started: Instant::now(),
        }
    }
}

fn start_warmup(
    mut bench: ResMut<Bench>,
    config: Res<BenchConfig>,
    q_loading: Query<(), (With<BenchModel>, With<ModelLoadTiming>)>,
) {
    if matches!(bench.phase, Phase::Loading) && q_loading.is_empty() {
        info!(
            "Benchmark: {:.0} s warmup, then {} runs of {:.0} s",
            config.warmup, config.runs, config.duration
        );
        bench.phase = Phase::Warmup { elapsed: 0. };
    }
}

fn record_frame_time(time: Res<Time<Real>>, config: Res<BenchConfig>, mut bench: ResMut<Bench>) {
    let delta = time.delta_secs();
    match &mut bench.phase {
        Phase::Warmup { elapsed } => {
            *elapsed += delta;
            if *elapsed >= config.warmup {
                info!("Benchmark run 1/{} started", config.runs);
                bench.phase = Phase::Running(BenchmarkRun::new(1));
            }
        }
        Phase::Running(run) => {
            run.updates += 1;
            if delta > 0. {
                run.elapsed += delta;
                run.frames.push(delta * 1000.);
            }
        }
        Phase::Loading | Phase::Done => {}
    }
}

fn orbit_camera(
    bench: Res<Bench>,
    config: Res<BenchConfig>,
    mut q_camera: Query<&mut Transform, With<BenchCamera>>,
) {
    // driven by elapsed time so every run covers exactly the same path
    let yaw = match &bench.phase {
        Phase::Warmup { elapsed } => TAU * elapsed / config.duration,
        Phase::Running(run) => TAU * (run.elapsed / config.duration).min(1.),
        Phase::Loading | Phase::Done => return,
    };
    for mut transform in &mut q_camera {
        *transform = orbit_transform(yaw, config.distance);
    }
}

fn finish_run(
    mut bench: ResMut<Bench>,
    csv: CsvContext,
    measure: MeasureModel,
    mut exit: MessageWriter<AppExit>,
) {
    let config = &csv.config;
    let Phase::Running(run) = &bench.phase else {
        return;
    };
    if run.elapsed < config.duration {
        return;
    }
    let next = if run.index < config.runs {
        info!("Benchmark run {}/{} started", run.index + 1, config.runs);
        Phase::Running(BenchmarkRun::new(run.index + 1))
    } else {
        Phase::Done
    };
    let Phase::Running(run) = std::mem::replace(&mut bench.phase, next) else {
        unreachable!()
    };

    if let Some(stats) = FrameStats::new(&run.frames) {
        let model = measure.measure();
        let summary = format!(
            "\n===== Benchmark run {}/{} ({:.1} s, one camera orbit) =====\n\
             frames            {}\n\
             average fps       {:.1}\n\
             median fps        {:.1}\n\
             1% low fps        {:.1}\n\
             0.1% low fps      {:.1}\n\
             frame time avg    {:.2} ms\n\
             frame time min    {:.2} ms\n\
             frame time p50    {:.2} ms\n\
             frame time p95    {:.2} ms\n\
             frame time p99    {:.2} ms\n\
             frame time max    {:.2} ms\n\
             frame time stddev {:.2} ms\n\
             wall-clock fps    {:.1} ({} updates, {} zero-length)\n\
             mesh instances    {}\n\
             unique meshes     {}\n\
             vertices          {}\n\
             triangles         {}\n\
             materials         {}\n\
             textures          {} ({:.2} MiB)\n\
             entities          {}",
            run.index,
            config.runs,
            run.elapsed,
            stats.frames,
            stats.average_fps,
            stats.median_fps(),
            stats.low_1_fps,
            stats.low_01_fps,
            stats.average_ms,
            stats.min_ms,
            stats.p50_ms,
            stats.p95_ms,
            stats.p99_ms,
            stats.max_ms,
            stats.stddev_ms,
            run.updates as f32 / run.started.elapsed().as_secs_f32(),
            run.updates,
            run.updates - stats.frames,
            model.mesh_instances,
            model.unique_meshes,
            model.vertices,
            model.triangles,
            model.materials,
            model.textures,
            model.texture_mib,
            model.entities,
        );
        let row = emit_csv(
            &csv,
            "rendering_fps.csv",
            FPS_CSV_HEADER,
            &[
                format!("{:.1}", run.elapsed),
                stats.frames.to_string(),
                format!("{:.1}", stats.average_fps),
                format!("{:.1}", stats.median_fps()),
                format!("{:.1}", stats.low_1_fps),
                format!("{:.1}", stats.low_01_fps),
                format!("{:.2}", stats.average_ms),
                format!("{:.2}", stats.min_ms),
                format!("{:.2}", stats.p50_ms),
                format!("{:.2}", stats.p95_ms),
                format!("{:.2}", stats.p99_ms),
                format!("{:.2}", stats.max_ms),
                format!("{:.2}", stats.stddev_ms),
                model.mesh_instances.to_string(),
                model.unique_meshes.to_string(),
                model.vertices.to_string(),
                model.triangles.to_string(),
                model.materials.to_string(),
                model.textures.to_string(),
                format!("{:.2}", model.texture_mib),
                model.entities.to_string(),
            ],
            &summary,
        );
        bench.rows.push(row);
    } else {
        warn!("Benchmark run {}/{} recorded no frames", run.index, config.runs);
    }

    if matches!(bench.phase, Phase::Done) {
        info!(
            "Benchmark finished, {} runs\n----- csv:\n{COMMON_CSV_HEADER},{FPS_CSV_HEADER}\n{}",
            config.runs,
            bench.rows.join("\n")
        );
        if config.exit_when_done && cfg!(not(target_arch = "wasm32")) {
            exit.write(AppExit::Success);
        }
    }
}

/// Everything the common CSV columns are made from
#[derive(SystemParam)]
pub struct CsvContext<'w, 's> {
    pub config: Res<'w, BenchConfig>,
    adapter: Res<'w, RenderAdapterInfo>,
    q_window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

impl CsvContext<'_, '_> {
    fn common_fields(&self) -> Vec<String> {
        let resolution = self
            .q_window
            .single()
            .map(|window| format!("{}x{}", window.physical_width(), window.physical_height()))
            .unwrap_or_default();
        let (browser, browser_version) = system_info::browser();
        vec![
            "bevy".to_string(),
            self.config.msaa.to_string(),
            if self.config.shadows { "on" } else { "off" }.to_string(),
            format!("{:.2}", self.config.distance),
            system_info::date(),
            system_info::PC_NAME.to_string(),
            system_info::os(),
            system_info::cpu_cores().to_string(),
            self.adapter.name.clone(),
            format!("{:?}", self.adapter.backend),
            browser,
            browser_version,
            system_info::BUILD.to_string(),
            resolution,
            self.config.model_id.clone(),
            self.config.model_name.clone(),
            self.config.model_version.clone(),
        ]
    }
}

/// Logs `summary` with the CSV row, natively also appends the row to `out/file_name`.
/// Returns the row.
pub fn emit_csv(
    csv: &CsvContext,
    file_name: &str,
    header: &str,
    fields: &[String],
    summary: &str,
) -> String {
    let mut all = csv.common_fields();
    all.extend_from_slice(fields);
    let row = system_info::csv_row(&all);
    let header = format!("{COMMON_CSV_HEADER},{header}");
    info!("{summary}\n----- csv: {file_name}\n{header}\n{row}");

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(dir) = &csv.config.out_dir {
        if let Err(error) = append_csv(std::path::Path::new(dir).join(file_name), &header, &row) {
            error!("can't write {file_name} to {dir}: {error}");
        }
    }
    row
}

/// Appends the row, writes the header first if the file is new
#[cfg(not(target_arch = "wasm32"))]
fn append_csv(path: std::path::PathBuf, header: &str, row: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    if file.metadata()?.len() == 0 {
        writeln!(file, "{header}")?;
    }
    writeln!(file, "{row}")
}

struct FrameStats {
    frames: usize,
    average_fps: f32,
    /// fps of the average of the slowest 1% of frames
    low_1_fps: f32,
    low_01_fps: f32,
    average_ms: f32,
    min_ms: f32,
    p50_ms: f32,
    p95_ms: f32,
    p99_ms: f32,
    max_ms: f32,
    stddev_ms: f32,
}

impl FrameStats {
    /// fps of the typical frame, closer to what the fps overlay shows than `average_fps`
    fn median_fps(&self) -> f32 {
        1000. / self.p50_ms
    }

    fn new(frame_times_ms: &[f32]) -> Option<Self> {
        if frame_times_ms.is_empty() {
            return None;
        }
        let mut sorted = frame_times_ms.to_vec();
        sorted.sort_by(f32::total_cmp);
        let n = sorted.len();
        let total: f32 = sorted.iter().sum();
        let average_ms = total / n as f32;
        let variance = sorted
            .iter()
            .map(|ms| (ms - average_ms).powi(2))
            .sum::<f32>()
            / n as f32;
        let percentile = |p: f32| sorted[((p / 100.) * (n - 1) as f32).round() as usize];
        let low_fps = |fraction: f32| {
            let count = ((n as f32 * fraction).ceil() as usize).max(1);
            let slowest = &sorted[n - count..];
            1000. / (slowest.iter().sum::<f32>() / count as f32)
        };
        Some(Self {
            frames: n,
            average_fps: 1000. * n as f32 / total,
            low_1_fps: low_fps(0.01),
            low_01_fps: low_fps(0.001),
            average_ms,
            min_ms: sorted[0],
            p50_ms: percentile(50.),
            p95_ms: percentile(95.),
            p99_ms: percentile(99.),
            max_ms: sorted[n - 1],
            stddev_ms: variance.sqrt(),
        })
    }
}
