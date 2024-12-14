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
    commands
        .spawn((
            SceneRoot(gltf_assets.building.clone()),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.25)),
            // ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            // PickableBundle::default(),
            // AvianPickable,
            // RigidBody::Static,
        ))
        .observe(add_tag);
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

fn add_tag(
    pick_hit: Trigger<Pointer<Down>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_entity = pick_hit.entity();

    match pick_hit.hit.position {
        Some(position) => {
            commands
                .spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.3))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.0, 0.0),
                        ..Default::default()
                    })),
                    GlobalTransform::from_xyz(position.x, position.y, position.z),
                ))
                .set_parent_in_place(target_entity);
        }
        None => (),
    };
}
