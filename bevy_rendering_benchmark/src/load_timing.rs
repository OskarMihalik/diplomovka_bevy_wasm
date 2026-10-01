//! How long opening the model takes, split into the same steps as bevy_diplomovka's building.rs
//! `log_model_load_timing`: download + parsing, mesh creation (scene spawn) and the first frame.

use bevy::app::AppExit;
use bevy::diagnostic::FrameCount;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAssetRoot, WorldInstanceReady};

use crate::bench::{CsvContext, emit_csv};
use crate::measure::mesh_counts;

const LOAD_CSV_HEADER: &str = "meshes,vertices,triangles,download_parse_ms,mesh_creation_ms,first_frame_ms,total_ms,frames";

pub struct LoadTimingPlugin;

impl Plugin for LoadTimingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_scene_ready)
            .add_systems(Update, log_model_load_timing);
    }
}

/// Removed once the model has been rendered for the first time, the benchmark starts then
#[derive(Component)]
pub struct ModelLoadTiming {
    started: Instant,
    started_frame: u32,
    /// end of the previous step
    last_step: Instant,
    download_parse_ms: Option<f64>,
    spawn_ms: Option<f64>,
    /// frame in which the scene was spawned into the world
    spawned_frame: Option<u32>,
}

impl ModelLoadTiming {
    /// Started when the model is requested, at startup or once it was picked on the page
    pub fn start(frame: u32) -> Self {
        Self {
            started: Instant::now(),
            started_frame: frame,
            last_step: Instant::now(),
            download_parse_ms: None,
            spawn_ms: None,
            spawned_frame: None,
        }
    }

    /// How long the step that just finished took, in ms
    fn step_ms(&mut self) -> f64 {
        let now = Instant::now();
        let step = now.duration_since(self.last_step);
        self.last_step = now;
        step.as_secs_f64() * 1000.
    }

    fn total_ms(&self) -> f64 {
        self.started.elapsed().as_secs_f64() * 1000.
    }
}

fn on_scene_ready(
    trigger: On<WorldInstanceReady>,
    mut q_timing: Query<&mut ModelLoadTiming>,
    frame: Res<FrameCount>,
) {
    let Ok(mut timing) = q_timing.get_mut(trigger.entity) else {
        return;
    };
    timing.spawned_frame = Some(frame.0);
    let spawn_ms = timing.step_ms();
    timing.spawn_ms = Some(spawn_ms);
    info!("model load: mesh creation (scene spawn) took {spawn_ms:.1} ms");
}

/// Steps are noticed once per frame, so they are precise to one frame time.
fn log_model_load_timing(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    frame: Res<FrameCount>,
    mut q_models: Query<(Entity, &WorldAssetRoot, &mut ModelLoadTiming)>,
    q_children: Query<&Children>,
    q_meshes: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    csv: CsvContext,
    mut exit: MessageWriter<AppExit>,
) {
    for (entity, root, mut timing) in &mut q_models {
        if timing.download_parse_ms.is_none() {
            if asset_server.load_state(&root.0).is_failed() {
                error!("model load failed: {:?}", asset_server.load_state(&root.0));
                exit.write(AppExit::error());
                return;
            }
            if asset_server.is_loaded_with_dependencies(&root.0) {
                let download_parse_ms = timing.step_ms();
                timing.download_parse_ms = Some(download_parse_ms);
                info!("model load: download + parsing took {download_parse_ms:.1} ms");
            }
        }
        // the frame the scene was spawned in has been rendered once the next frame starts
        if !timing
            .spawned_frame
            .is_some_and(|spawned| frame.0 > spawned)
        {
            continue;
        }
        let first_frame_ms = timing.step_ms();

        let mut mesh_count = 0;
        let mut vertices = 0;
        let mut triangles = 0;
        for mesh in q_meshes.iter_many(q_children.iter_descendants(entity)) {
            mesh_count += 1;
            if let Some(mesh) = meshes.get(mesh) {
                let (v, t) = mesh_counts(mesh);
                vertices += v;
                triangles += t;
            }
        }

        let download_parse_ms = timing.download_parse_ms.unwrap_or_default();
        let spawn_ms = timing.spawn_ms.unwrap_or_default();
        let total_ms = timing.total_ms();
        let frames = frame.0.saturating_sub(timing.started_frame);

        let summary = format!(
            "\n===== model load: {} =====\n\
             download + parsing  {download_parse_ms:>9.1} ms\n\
             mesh creation       {spawn_ms:>9.1} ms\n\
             first frame render  {first_frame_ms:>9.1} ms\n\
             total               {total_ms:>9.1} ms\n\
             meshes              {mesh_count:>9}\n\
             vertices            {vertices:>9}\n\
             triangles           {triangles:>9}\n\
             frames until shown  {frames:>9}",
            csv.config.model_name,
        );
        emit_csv(
            &csv,
            "rendering_load.csv",
            LOAD_CSV_HEADER,
            &[
                mesh_count.to_string(),
                vertices.to_string(),
                triangles.to_string(),
                format!("{download_parse_ms:.1}"),
                format!("{spawn_ms:.1}"),
                format!("{first_frame_ms:.1}"),
                format!("{total_ms:.1}"),
                frames.to_string(),
            ],
            &summary,
        );
        commands.entity(entity).remove::<ModelLoadTiming>();
    }
}
