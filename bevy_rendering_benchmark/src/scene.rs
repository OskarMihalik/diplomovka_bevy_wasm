//! The same scene bevy_diplomovka shows when a model is opened (building.rs `setup_scene` and
//! `react_to_model_change`): one point light, the model at the origin scaled to 0.25 and a camera
//! orbiting the origin. PanOrbitCamera is replaced by setting the transform directly, its
//! smoothing would make the camera lag behind the benchmark's yaw.

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use crate::config::BenchConfig;
use crate::load_timing::ModelLoadTiming;
use crate::ModelAssetPath;

/// Where bevy_diplomovka spawns its PanOrbitCamera, the orbit pitch comes from it
const CAMERA_START: Vec3 = Vec3::new(0.0, 1.5, 5.0);
/// Orbit radius of bevy_diplomovka, `CAMERA_START.length()`
pub const DEFAULT_DISTANCE: f32 = 5.220153;
const MODEL_SCALE: f32 = 0.25;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene).add_systems(
            Update,
            spawn_model.run_if(resource_added::<ModelAssetPath>),
        );
    }
}

/// The loaded model
#[derive(Component)]
pub struct BenchModel;

#[derive(Component)]
pub struct BenchCamera;

fn setup_scene(mut commands: Commands, config: Res<BenchConfig>) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: config.shadows,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        BenchCamera,
        Camera3d::default(),
        config.msaa(),
        orbit_transform(0., config.distance),
    ));
}

/// At startup, or once a model was picked on the page
fn spawn_model(
    mut commands: Commands,
    model_path: Res<ModelAssetPath>,
    asset_server: Res<AssetServer>,
    frame: Res<FrameCount>,
) {
    info!("model load: request started, {}", model_path.0);
    let scene = asset_server.load(format!("{}#Scene0", model_path.0));
    commands.spawn((
        BenchModel,
        WorldAssetRoot(scene),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(MODEL_SCALE)),
        ModelLoadTiming::start(frame.0),
    ));
}

/// Camera transform at `yaw` radians around the Y axis and `radius` meters from the origin,
/// same math as PanOrbitCamera
pub fn orbit_transform(yaw: f32, radius: f32) -> Transform {
    let pitch = (CAMERA_START.y / CAMERA_START.length()).asin();
    let start_yaw = CAMERA_START.x.atan2(CAMERA_START.z);
    let rotation = Quat::from_axis_angle(Vec3::Y, start_yaw + yaw)
        * Quat::from_axis_angle(Vec3::X, -pitch);
    Transform {
        translation: rotation * Vec3::new(0.0, 0.0, radius),
        rotation,
        ..default()
    }
}
