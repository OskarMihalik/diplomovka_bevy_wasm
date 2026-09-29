//! Debug overlays built on `bevy_dev_tools`, used to inspect how different glb models perform.
//! Only compiled with the `debug_tools` cargo feature, it pulls in bevy_ui and grows the wasm.
//!
//! Keys:
//! - F1: cycle render debug overlay (depth / normals, needs prepasses from F3)
//! - F2: cycle render debug overlay opacity
//! - F3: toggle depth + normal prepass on the 3d cameras
//! - F4: toggle wireframe (native only, not supported on WebGL)
//! - F5: toggle bounding boxes of all meshes
//! - F6: toggle ground grid
//! - F7: toggle fps counter and diagnostics windows
//! - F8: toggle camera auto orbit
//! - F9: run a benchmark, one full camera orbit, and print the results to the console
//!
//! The app runs uncapped (no reactive / low power update mode, no vsync on native) so the
//! numbers measure the model and not the frame limiter. Every few seconds a short frame time
//! summary is printed to the console as well.

use bevy::camera::primitives::Aabb;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::dev_tools::diagnostics_overlay::{
    DiagnosticsOverlay, DiagnosticsOverlayItem, DiagnosticsOverlayPlane, DiagnosticsOverlayPlugin,
    DiagnosticsOverlayStatistic,
};
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin};
use bevy::diagnostic::{
    Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore, EntityCountDiagnosticsPlugin,
    RegisterDiagnostic,
};
use bevy::gizmos::aabb::AabbGizmoConfigGroup;
use bevy::mesh::PrimitiveTopology;
use bevy::pbr::diagnostic::MaterialAllocatorDiagnosticPlugin;
use bevy::prelude::*;
use bevy::render::diagnostic::MeshAllocatorDiagnosticPlugin;
use bevy::time::common_conditions::on_timer;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy::winit::WinitSettings;
use bevy_panorbit_camera::PanOrbitCamera;
use std::collections::HashSet;
use std::f32::consts::TAU;
use std::time::Duration;

const MESH_INSTANCES: DiagnosticPath = DiagnosticPath::const_new("model/mesh_instances");
const UNIQUE_MESHES: DiagnosticPath = DiagnosticPath::const_new("model/unique_meshes");
const VERTICES: DiagnosticPath = DiagnosticPath::const_new("model/vertices");
const TRIANGLES: DiagnosticPath = DiagnosticPath::const_new("model/triangles");
const MATERIALS: DiagnosticPath = DiagnosticPath::const_new("model/materials");
const TEXTURES: DiagnosticPath = DiagnosticPath::const_new("model/textures");
const TEXTURE_MEMORY: DiagnosticPath = DiagnosticPath::const_new("model/texture_memory");
const LIGHTS: DiagnosticPath = DiagnosticPath::const_new("model/lights");
const SIZE_X: DiagnosticPath = DiagnosticPath::const_new("model/size_x");
const SIZE_Y: DiagnosticPath = DiagnosticPath::const_new("model/size_y");
const SIZE_Z: DiagnosticPath = DiagnosticPath::const_new("model/size_z");

/// How often the rolling frame time summary is printed
const SUMMARY_INTERVAL: Duration = Duration::from_secs(5);
/// How long the F9 benchmark takes, the camera does one full orbit in this time
const BENCHMARK_DURATION: f32 = 60.;
/// Auto orbit speed in radians per second
const ORBIT_SPEED: f32 = TAU / BENCHMARK_DURATION;

pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont::from_font_size(16.),
                    ..default()
                },
            },
            DiagnosticsOverlayPlugin,
            // RenderDebugOverlayPlugin (F1/F2) is already part of DefaultPlugins with bevy_dev_tools
            InfiniteGridPlugin,
            EntityCountDiagnosticsPlugin::default(),
            MeshAllocatorDiagnosticPlugin,
            MaterialAllocatorDiagnosticPlugin::<StandardMaterial>::default(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(bevy::pbr::wireframe::WireframePlugin::default());

        for (path, suffix) in [
            (MESH_INSTANCES, ""),
            (UNIQUE_MESHES, ""),
            (VERTICES, ""),
            (TRIANGLES, ""),
            (MATERIALS, ""),
            (TEXTURES, ""),
            (TEXTURE_MEMORY, " MiB"),
            (LIGHTS, ""),
            (SIZE_X, " m"),
            (SIZE_Y, " m"),
            (SIZE_Z, " m"),
        ] {
            app.register_diagnostic(Diagnostic::new(path).with_suffix(suffix));
        }

        app.init_resource::<FrameTimes>()
            .init_resource::<AutoOrbit>()
            // .add_systems(Startup, spawn_overlays)
            // after main.rs sets up the reactive update mode in Startup
            .add_systems(PostStartup, uncap_frame_rate)
            .add_systems(
                Update,
                (
                    handle_input,
                    measure_model.run_if(on_timer(Duration::from_millis(500))),
                    (record_frame_time, orbit_camera, finish_benchmark).chain(),
                    // print_summary.run_if(on_timer(SUMMARY_INTERVAL)),
                ),
            );
    }
}

fn item(path: DiagnosticPath, precision: usize) -> DiagnosticsOverlayItem {
    DiagnosticsOverlayItem {
        path,
        statistic: DiagnosticsOverlayStatistic::Value,
        precision,
    }
}

fn spawn_overlays(mut commands: Commands) {
    commands.spawn(DiagnosticsOverlay::new(
        "Model",
        vec![
            item(MESH_INSTANCES, 0),
            item(UNIQUE_MESHES, 0),
            item(VERTICES, 0),
            item(TRIANGLES, 0),
            item(MATERIALS, 0),
            item(TEXTURES, 0),
            item(TEXTURE_MEMORY, 2),
            item(LIGHTS, 0),
            item(SIZE_X, 2),
            item(SIZE_Y, 2),
            item(SIZE_Z, 2),
            item(EntityCountDiagnosticsPlugin::ENTITY_COUNT, 0),
        ],
    ));
    commands.spawn(DiagnosticsOverlay::fps());
    commands.spawn(DiagnosticsOverlay::mesh_and_standard_material());
}

/// Measures what is currently visible, so the numbers describe the loaded model and not
/// every asset that happens to be in memory.
fn measure_model(
    mut diagnostics: Diagnostics,
    q_meshes: Query<(
        &Mesh3d,
        Option<&MeshMaterial3d<StandardMaterial>>,
        &InheritedVisibility,
        Option<&Aabb>,
        &GlobalTransform,
    )>,
    q_lights: Query<(), Or<(With<PointLight>, With<SpotLight>, With<DirectionalLight>)>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
) {
    let mut instances = 0usize;
    let mut vertices = 0usize;
    let mut triangles = 0usize;
    let mut unique_meshes = HashSet::new();
    let mut used_materials = HashSet::new();
    let mut bounds_min = Vec3::splat(f32::MAX);
    let mut bounds_max = Vec3::splat(f32::MIN);

    for (mesh_handle, material, visibility, aabb, transform) in &q_meshes {
        if !visibility.get() {
            continue;
        }
        instances += 1;
        unique_meshes.insert(mesh_handle.id());
        if let Some(material) = material {
            used_materials.insert(material.id());
        }
        if let Some(mesh) = meshes.get(mesh_handle) {
            let (v, t) = mesh_counts(mesh);
            vertices += v;
            triangles += t;
        }
        if let Some(aabb) = aabb {
            let (min, max) = world_aabb(aabb, transform);
            bounds_min = bounds_min.min(min);
            bounds_max = bounds_max.max(max);
        }
    }

    let mut textures = HashSet::new();
    for material in used_materials.iter().filter_map(|id| materials.get(*id)) {
        for texture in [
            &material.base_color_texture,
            &material.emissive_texture,
            &material.metallic_roughness_texture,
            &material.normal_map_texture,
            &material.occlusion_texture,
        ]
        .into_iter()
        .flatten()
        {
            textures.insert(texture.id());
        }
    }
    let texture_bytes: u64 = textures
        .iter()
        .filter_map(|id| images.get(*id))
        .map(texture_size)
        .sum();

    let size = if instances > 0 && bounds_min.x <= bounds_max.x {
        bounds_max - bounds_min
    } else {
        Vec3::ZERO
    };

    diagnostics.add_measurement(&MESH_INSTANCES, || instances as f64);
    diagnostics.add_measurement(&UNIQUE_MESHES, || unique_meshes.len() as f64);
    diagnostics.add_measurement(&VERTICES, || vertices as f64);
    diagnostics.add_measurement(&TRIANGLES, || triangles as f64);
    diagnostics.add_measurement(&MATERIALS, || used_materials.len() as f64);
    diagnostics.add_measurement(&TEXTURES, || textures.len() as f64);
    diagnostics.add_measurement(&TEXTURE_MEMORY, || texture_bytes as f64 / (1024. * 1024.));
    diagnostics.add_measurement(&LIGHTS, || q_lights.iter().count() as f64);
    diagnostics.add_measurement(&SIZE_X, || size.x as f64);
    diagnostics.add_measurement(&SIZE_Y, || size.y as f64);
    diagnostics.add_measurement(&SIZE_Z, || size.z as f64);
}

/// Vertex and triangle count, zero if the mesh data only lives in the render world.
fn mesh_counts(mesh: &Mesh) -> (usize, usize) {
    let Ok(positions) = mesh.try_attribute(Mesh::ATTRIBUTE_POSITION) else {
        return (0, 0);
    };
    let vertices = positions.len();
    let indices = match mesh.try_indices_option() {
        Ok(Some(indices)) => indices.len(),
        _ => vertices,
    };
    let triangles = match mesh.primitive_topology() {
        PrimitiveTopology::TriangleList => indices / 3,
        PrimitiveTopology::TriangleStrip => indices.saturating_sub(2),
        _ => 0,
    };
    (vertices, triangles)
}

fn world_aabb(aabb: &Aabb, transform: &GlobalTransform) -> (Vec3, Vec3) {
    let center = Vec3::from(aabb.center);
    let half = Vec3::from(aabb.half_extents);
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for corner in 0..8 {
        let sign = Vec3::new(
            if corner & 1 == 0 { -1. } else { 1. },
            if corner & 2 == 0 { -1. } else { 1. },
            if corner & 4 == 0 { -1. } else { 1. },
        );
        let point = transform.transform_point(center + half * sign);
        min = min.min(point);
        max = max.max(point);
    }
    (min, max)
}

/// GPU size estimate from the texture descriptor, the pixel data itself may already be dropped
/// from the main world after upload.
fn texture_size(image: &Image) -> u64 {
    let descriptor = &image.texture_descriptor;
    let format = descriptor.format;
    let (block_w, block_h) = format.block_dimensions();
    let block_bytes = format.block_copy_size(None).unwrap_or(4) as u64;
    let mut width = descriptor.size.width;
    let mut height = descriptor.size.height;
    let layers = descriptor.size.depth_or_array_layers as u64;
    let mut total = 0;
    for _ in 0..descriptor.mip_level_count.max(1) {
        let blocks_x = width.div_ceil(block_w) as u64;
        let blocks_y = height.div_ceil(block_h) as u64;
        total += blocks_x * blocks_y * block_bytes * layers;
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }
    total
}

fn handle_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gizmo_config: ResMut<GizmoConfigStore>,
    mut fps_config: ResMut<FpsOverlayConfig>,
    q_cameras: Query<(Entity, Has<DepthPrepass>), With<Camera3d>>,
    q_grid: Query<Entity, With<InfiniteGrid>>,
    mut q_overlay_plane: Query<&mut Node, With<DiagnosticsOverlayPlane>>,
    mut auto_orbit: ResMut<AutoOrbit>,
    mut frame_times: ResMut<FrameTimes>,
    q_orbit_camera: Query<&PanOrbitCamera>,
    #[cfg(not(target_arch = "wasm32"))] mut wireframe: ResMut<
        bevy::pbr::wireframe::WireframeConfig,
    >,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        for (camera, has_prepass) in &q_cameras {
            if has_prepass {
                commands
                    .entity(camera)
                    .remove::<(DepthPrepass, NormalPrepass)>();
            } else {
                commands
                    .entity(camera)
                    .insert((DepthPrepass, NormalPrepass));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard.just_pressed(KeyCode::F4) {
        wireframe.global = !wireframe.global;
    }
    if keyboard.just_pressed(KeyCode::F5) {
        let (_, aabb) = gizmo_config.config_mut::<AabbGizmoConfigGroup>();
        aabb.draw_all = !aabb.draw_all;
    }
    if keyboard.just_pressed(KeyCode::F6) {
        if q_grid.is_empty() {
            commands.spawn(InfiniteGrid);
        } else {
            for grid in &q_grid {
                commands.entity(grid).despawn();
            }
        }
    }
    if keyboard.just_pressed(KeyCode::F7) {
        fps_config.enabled = !fps_config.enabled;
        for mut node in &mut q_overlay_plane {
            node.display = if fps_config.enabled {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
    if keyboard.just_pressed(KeyCode::F8) {
        auto_orbit.0 = !auto_orbit.0;
    }
    if keyboard.just_pressed(KeyCode::F9) && frame_times.benchmark.is_none() {
        let start_yaw = q_orbit_camera
            .iter()
            .next()
            .map(|camera| camera.target_yaw)
            .unwrap_or_default();
        info!("Benchmark started, {BENCHMARK_DURATION} s");
        frame_times.benchmark = Some(BenchmarkRun {
            elapsed: 0.,
            start_yaw,
            frames: Vec::new(),
        });
    }
}

fn uncap_frame_rate(
    mut winit: ResMut<WinitSettings>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    *winit = WinitSettings::continuous();
    // browsers always sync to the display through requestAnimationFrame, this only matters natively
    // for mut window in &mut q_window {
    //     window.present_mode = PresentMode::AutoNoVsync;
    // }
}

#[derive(Resource, Default)]
struct AutoOrbit(bool);

#[derive(Resource, Default)]
struct FrameTimes {
    /// frame times in ms since the last summary
    window: Vec<f32>,
    benchmark: Option<BenchmarkRun>,
}

struct BenchmarkRun {
    elapsed: f32,
    start_yaw: f32,
    frames: Vec<f32>,
}

fn record_frame_time(time: Res<Time<Real>>, mut frame_times: ResMut<FrameTimes>) {
    let frame_ms = time.delta_secs() * 1000.;
    if frame_ms <= 0. {
        return;
    }
    frame_times.window.push(frame_ms);
    if let Some(run) = &mut frame_times.benchmark {
        run.elapsed += time.delta_secs();
        run.frames.push(frame_ms);
    }
}

fn orbit_camera(
    time: Res<Time<Real>>,
    auto_orbit: Res<AutoOrbit>,
    frame_times: Res<FrameTimes>,
    mut q_camera: Query<&mut PanOrbitCamera>,
) {
    for mut camera in &mut q_camera {
        if let Some(run) = &frame_times.benchmark {
            // driven by elapsed time so every run covers exactly the same path
            camera.target_yaw = run.start_yaw + TAU * (run.elapsed / BENCHMARK_DURATION).min(1.);
        } else if auto_orbit.0 {
            camera.target_yaw += ORBIT_SPEED * time.delta_secs();
        }
    }
}

fn finish_benchmark(
    mut frame_times: ResMut<FrameTimes>,
    diagnostics: Res<DiagnosticsStore>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    if !frame_times
        .benchmark
        .as_ref()
        .is_some_and(|run| run.elapsed >= BENCHMARK_DURATION)
    {
        return;
    }
    let Some(run) = frame_times.benchmark.take() else {
        return;
    };
    let Some(stats) = FrameStats::new(&run.frames) else {
        warn!("Benchmark recorded no frames");
        return;
    };
    let value = |path: &DiagnosticPath| {
        diagnostics
            .get(path)
            .and_then(|diagnostic| diagnostic.value())
            .unwrap_or(0.)
    };
    let resolution = q_window
        .single()
        .map(|window| format!("{}x{}", window.physical_width(), window.physical_height()))
        .unwrap_or_default();

    info!(
        "\n===== Benchmark ({:.1} s, one camera orbit) =====\n\
         resolution        {resolution}\n\
         frames            {}\n\
         average fps       {:.1}\n\
         1% low fps        {:.1}\n\
         0.1% low fps      {:.1}\n\
         frame time avg    {:.2} ms\n\
         frame time min    {:.2} ms\n\
         frame time p50    {:.2} ms\n\
         frame time p95    {:.2} ms\n\
         frame time p99    {:.2} ms\n\
         frame time max    {:.2} ms\n\
         frame time stddev {:.2} ms\n\
         mesh instances    {:.0}\n\
         unique meshes     {:.0}\n\
         vertices          {:.0}\n\
         triangles         {:.0}\n\
         materials         {:.0}\n\
         textures          {:.0} ({:.2} MiB)\n\
         entities          {:.0}\n\
         ==============================================",
        run.elapsed,
        stats.frames,
        stats.average_fps,
        stats.low_1_fps,
        stats.low_01_fps,
        stats.average_ms,
        stats.min_ms,
        stats.p50_ms,
        stats.p95_ms,
        stats.p99_ms,
        stats.max_ms,
        stats.stddev_ms,
        value(&MESH_INSTANCES),
        value(&UNIQUE_MESHES),
        value(&VERTICES),
        value(&TRIANGLES),
        value(&MATERIALS),
        value(&TEXTURES),
        value(&TEXTURE_MEMORY),
        value(&EntityCountDiagnosticsPlugin::ENTITY_COUNT),
    );
}

fn print_summary(mut frame_times: ResMut<FrameTimes>) {
    let frames = std::mem::take(&mut frame_times.window);
    let Some(stats) = FrameStats::new(&frames) else {
        return;
    };
    info!(
        "fps avg {:.1} | 1% low {:.1} | frame time avg {:.2} ms, p50 {:.2}, p95 {:.2}, p99 {:.2}, max {:.2} ({} frames)",
        stats.average_fps,
        stats.low_1_fps,
        stats.average_ms,
        stats.p50_ms,
        stats.p95_ms,
        stats.p99_ms,
        stats.max_ms,
        stats.frames,
    );
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
