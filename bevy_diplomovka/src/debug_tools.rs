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

use bevy::camera::primitives::Aabb;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::dev_tools::diagnostics_overlay::{
    DiagnosticsOverlay, DiagnosticsOverlayItem, DiagnosticsOverlayPlane, DiagnosticsOverlayPlugin,
    DiagnosticsOverlayStatistic,
};
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin};
use bevy::diagnostic::{
    Diagnostic, DiagnosticPath, Diagnostics, EntityCountDiagnosticsPlugin, RegisterDiagnostic,
};
use bevy::gizmos::aabb::AabbGizmoConfigGroup;
use bevy::mesh::PrimitiveTopology;
use bevy::pbr::diagnostic::MaterialAllocatorDiagnosticPlugin;
use bevy::prelude::*;
use bevy::render::diagnostic::MeshAllocatorDiagnosticPlugin;
use bevy::time::common_conditions::on_timer;
use std::collections::HashSet;
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

        app.add_systems(Startup, spawn_overlays).add_systems(
            Update,
            (
                handle_input,
                measure_model.run_if(on_timer(Duration::from_millis(500))),
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
}
