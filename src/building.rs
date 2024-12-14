use bevy::prelude::*;

use crate::{loading::GltfAssets, GameState};

pub struct BuildingPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_building, setup_scene));
    }
}

fn spawn_building(mut commands: Commands, gltf_assets: Res<GltfAssets>) {
    commands.spawn((
        // SceneBundle {
        //     scene: gltf_assets.building.clone(),
        //     transform: Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.25)),
        //     ..default()
        // },
        SceneRoot(gltf_assets.building.clone()),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.25)),
        // ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
        // PickableBundle::default(),
        // On::<Pointer<Down>>::send_event::<AddTag>(),
        // AvianPickable,
        // RigidBody::Static,
    ));
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
