//! Bench mode for rendering_runner (only with the `debug_tools` feature): opens one model and
//! runs the F9 benchmark without anyone clicking. It takes the same steps as a user, through the
//! app's own events: log in, select the project, open the model; the benchmark starts after a
//! warmup. See rendering_runner/PLAN.md, section 7.
//!
//! Options, `--key value` natively and `?key=value` on the web:
//! - `bench_model`   id of the model to open, turns bench mode on
//! - `bench_project` id of the model's project
//! - `email`, `password` the account to log in with
//! - `runs`          measured runs, default 5; 0 only measures the load
//! - `duration`      seconds of one run, the camera does one orbit in this time, default 60
//! - `warmup`        seconds between the model being shown and the first run, default 5
//! - `resolution`    window size in physical pixels, default 2088x1550 in bench mode
//! - `distance`      orbit radius of the camera in m, default the app's own 5.22 (the model is
//!                   scaled to 0.25); the same orbit as bevy_rendering_benchmark's `distance`
//! - `pc`            name of the computer in the CSV rows, default `PC_NAME` at build time
//! - `out`           native: folder to append fps.csv and load.csv to
//! - `exit`          native: `off` keeps the app open when the benchmark is done
//! - `report`        web: url the rows are posted to, read by index.html
//!
//! Without `bench_model` the app works as usual, `runs` and `duration` still apply to F9.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use dto::auth::LoginDto;

use crate::api::LoginEvent;
use crate::building::{
    ModelData, ModelLoadTiming, ProjectData, ThisModelIsSelected, ThisProjectIsSelected,
};
use crate::debug_tools::{BenchmarkRun, FrameTimes};
use crate::gui::gui::ShowErrorEvent;
use crate::{report, system_info, GameState};

#[derive(Resource, Clone, Debug)]
pub struct BenchConfig {
    pub model: Option<i32>,
    pub project: Option<i32>,
    pub email: String,
    pub password: String,
    /// 0 = only the load is measured
    pub runs: usize,
    pub duration: f32,
    pub warmup: f32,
    /// fixed window size in physical pixels, otherwise the window fits the page as usual
    pub resolution: Option<(u32, u32)>,
    /// orbit radius of the camera, otherwise the app's own
    pub distance: Option<f32>,
    pub exit_when_done: bool,
}

impl Default for BenchConfig {
    /// F9 without bench mode: 5 runs of 60 s
    fn default() -> Self {
        Self {
            model: None,
            project: None,
            email: String::new(),
            password: String::new(),
            runs: 5,
            duration: 60.,
            warmup: 5.,
            resolution: None,
            distance: None,
            exit_when_done: true,
        }
    }
}

impl BenchConfig {
    /// From the command line or the page url. Also sets `pc` and `out` for the CSV rows.
    pub fn from_env() -> Result<Self, String> {
        let args = args()?;
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
        let id = |key: &str| -> Result<Option<i32>, String> {
            get(key)
                .map(|v| {
                    v.parse()
                        .map_err(|_| format!("`{key}` must be an id, got `{v}`"))
                })
                .transpose()
        };

        let model = id("bench_model")?;
        let project = id("bench_project")?;
        if model.is_some() && project.is_none() {
            return Err("`bench_model` needs `bench_project`, the id of the model's project".into());
        }
        let resolution = match get("resolution") {
            None => model.map(|_| (2088, 1550)),
            Some(resolution) => Some(
                resolution
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                    .ok_or(format!(
                        "`resolution` must look like 1920x1080, got `{resolution}`"
                    ))?,
            ),
        };
        let exit_when_done = match get("exit") {
            None | Some("on" | "true" | "1") => true,
            Some("off" | "false" | "0") => false,
            Some(other) => return Err(format!("`exit` must be `on` or `off`, got `{other}`")),
        };
        if let Some(pc) = get("pc") {
            system_info::set_pc(pc);
        }
        if let Some(out) = get("out") {
            report::set_out_dir(out);
        }

        Ok(Self {
            model,
            project,
            email: get("email").unwrap_or_default().to_string(),
            password: get("password").unwrap_or_default().to_string(),
            // a negative number saturates to 0
            runs: number("runs", 5.)? as usize,
            duration: number("duration", 60.)?.max(1.),
            warmup: number("warmup", 5.)?.max(0.),
            resolution,
            distance: get("distance")
                .map(|_| number("distance", 0.).map(|d| d.max(0.01)))
                .transpose()?,
            exit_when_done,
        })
    }

    pub fn active(&self) -> bool {
        self.model.is_some()
    }
}

/// `--key value` or `--key=value` pairs
#[cfg(not(target_arch = "wasm32"))]
fn args() -> Result<Vec<(String, String)>, String> {
    let mut args = std::env::args().skip(1);
    let mut pairs = Vec::new();
    while let Some(arg) = args.next() {
        let Some(key) = arg.strip_prefix("--") else {
            return Err(format!("unexpected argument `{arg}`, options are `--key value`"));
        };
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

/// `?key=value&...` from the page url
#[cfg(target_arch = "wasm32")]
fn args() -> Result<Vec<(String, String)>, String> {
    let search = web_sys::window()
        .ok_or("no window")?
        .location()
        .search()
        .unwrap_or_default();
    let decode = |s: &str| {
        js_sys::decode_uri_component(&s.replace('+', " "))
            .map(String::from)
            .unwrap_or_else(|_| s.to_string())
    };
    Ok(search
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (decode(key), decode(value))
        })
        .collect())
}

pub struct BenchModePlugin;

impl Plugin for BenchModePlugin {
    fn build(&self, app: &mut App) {
        // main.rs inserts the parsed config before, this only fills in the defaults otherwise
        app.init_resource::<BenchConfig>()
            .init_resource::<BenchProgress>()
            .add_systems(OnEnter(GameState::Auth), log_in)
            .add_systems(Update, set_camera_distance)
            .add_systems(
                Update,
                (select_project, select_model)
                    .chain()
                    .run_if(in_state(GameState::SelectingProjectAndModel)),
            )
            .add_systems(
                Update,
                start_benchmark.run_if(in_state(GameState::ViewingModel)),
            )
            .add_observer(fail_on_shown_error);
    }
}

#[derive(Resource, Default)]
struct BenchProgress {
    distance_set: bool,
    project_selected: bool,
    model_selected: bool,
    /// seconds since the model was shown
    warmup_elapsed: f32,
    started: bool,
    failed: bool,
}

/// The app's camera starts at (0, 1.5, 5) looking at the model, like bevy_rendering_benchmark's,
/// so changing only the radius gives the same orbit as its `distance`
fn set_camera_distance(
    config: Res<BenchConfig>,
    mut progress: ResMut<BenchProgress>,
    mut q_camera: Query<&mut PanOrbitCamera>,
) {
    let Some(distance) = config.distance else {
        return;
    };
    if progress.distance_set || q_camera.is_empty() {
        return;
    }
    for mut camera in &mut q_camera {
        // before its first update PanOrbitCamera takes `radius` as the start, afterwards
        // `force_update` applies it at once instead of zooming there smoothly
        camera.radius = Some(distance);
        camera.target_radius = distance;
        camera.force_update = true;
    }
    info!("bench mode: camera distance {distance} m");
    progress.distance_set = true;
}

/// What the login button does, with the account from the options
fn log_in(config: Res<BenchConfig>, mut commands: Commands) {
    if !config.active() {
        return;
    }
    info!("bench mode: logging in as {}", config.email);
    commands.trigger(LoginEvent {
        dto: LoginDto {
            email: config.email.clone(),
            password: config.password.clone(),
        },
    });
}

/// What clicking "Select" on the project does, once the projects are loaded
fn select_project(
    mut commands: Commands,
    config: Res<BenchConfig>,
    mut progress: ResMut<BenchProgress>,
    q_projects: Query<(Entity, &ProjectData)>,
    mut exit: MessageWriter<AppExit>,
) {
    if !config.active() || progress.project_selected || progress.failed || q_projects.is_empty() {
        return;
    }
    match q_projects
        .iter()
        .find(|(_, project)| Some(project.dto.id) == config.project)
    {
        Some((entity, project)) => {
            info!("bench mode: project {} ({})", project.dto.id, project.dto.name);
            commands.entity(entity).insert(ThisProjectIsSelected {});
            progress.project_selected = true;
        }
        None => fail(
            &format!("project {:?} isn't one of {}'s projects", config.project, config.email),
            &config,
            &mut progress,
            &mut exit,
        ),
    }
}

/// What clicking "Open" on the model does, once the project's models are loaded
fn select_model(
    mut commands: Commands,
    config: Res<BenchConfig>,
    mut progress: ResMut<BenchProgress>,
    q_models: Query<(Entity, &ModelData)>,
    mut exit: MessageWriter<AppExit>,
) {
    if !progress.project_selected || progress.model_selected || progress.failed {
        return;
    }
    let mut in_project = q_models
        .iter()
        .filter(|(_, model)| Some(model.dto.project_id) == config.project)
        .peekable();
    if in_project.peek().is_none() {
        return;
    }
    match in_project.find(|(_, model)| Some(model.dto.id) == config.model) {
        Some((entity, model)) => {
            info!("bench mode: model {} ({})", model.dto.id, model.dto.name);
            commands.entity(entity).insert(ThisModelIsSelected {});
            progress.model_selected = true;
        }
        None => fail(
            &format!("model {:?} isn't in project {:?}", config.model, config.project),
            &config,
            &mut progress,
            &mut exit,
        ),
    }
}

/// Once the load row was logged: done for `runs` = 0, else the F9 benchmark after the warmup
fn start_benchmark(
    time: Res<Time<Real>>,
    config: Res<BenchConfig>,
    mut progress: ResMut<BenchProgress>,
    asset_server: Res<AssetServer>,
    q_model: Query<(&WorldAssetRoot, Has<ModelLoadTiming>), With<ThisModelIsSelected>>,
    q_camera: Query<&PanOrbitCamera>,
    mut frame_times: ResMut<FrameTimes>,
    mut exit: MessageWriter<AppExit>,
) {
    if !config.active() || progress.started || progress.failed {
        return;
    }
    let Ok((root, loading)) = q_model.single() else {
        return;
    };
    if asset_server.load_state(&root.0).is_failed() {
        let state = asset_server.load_state(&root.0);
        fail(&format!("model load failed: {state:?}"), &config, &mut progress, &mut exit);
        return;
    }
    // building.rs removes ModelLoadTiming once the load row is logged
    if loading {
        return;
    }
    if config.runs == 0 {
        info!("Benchmark finished, load only");
        progress.started = true;
        report::done();
        if config.exit_when_done && cfg!(not(target_arch = "wasm32")) {
            exit.write(AppExit::Success);
        }
        return;
    }
    progress.warmup_elapsed += time.delta_secs();
    if progress.warmup_elapsed < config.warmup {
        return;
    }
    let start_yaw = q_camera
        .iter()
        .next()
        .map(|camera| camera.target_yaw)
        .unwrap_or_default();
    info!(
        "Benchmark started, {} runs of {} s",
        config.runs, config.duration
    );
    frame_times.benchmark = Some(BenchmarkRun::new(1, start_yaw));
    progress.started = true;
}

/// A failed login or request shows a toast, in bench mode it ends the benchmark
fn fail_on_shown_error(
    trigger: On<ShowErrorEvent>,
    config: Res<BenchConfig>,
    mut progress: ResMut<BenchProgress>,
    mut exit: MessageWriter<AppExit>,
) {
    if config.active() {
        fail(&trigger.event().message, &config, &mut progress, &mut exit);
    }
}

fn fail(
    message: &str,
    config: &BenchConfig,
    progress: &mut BenchProgress,
    exit: &mut MessageWriter<AppExit>,
) {
    if progress.failed {
        return;
    }
    progress.failed = true;
    error!("bench mode: {message}");
    report::error(message);
    if config.exit_when_done && cfg!(not(target_arch = "wasm32")) {
        exit.write(AppExit::error());
    }
}
