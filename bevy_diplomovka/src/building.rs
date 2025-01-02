use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use dto::default::{NewTagDto, TagDto};

use crate::{
    api::{CreateNewTagEvent, GetTagsEvent},
    loading::GltfAssets,
    utils::LogEntityComponents,
    GameState,
};

#[derive(Component)]
pub struct TagData {
    pub dto: TagDto,
}

pub struct BuildingPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_building, setup_scene))
            .add_observer(rebuild_tags);
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

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera::default(),
    ));
}

fn add_tag(
    pick_hit: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&SceneRoot, &Transform, &Children)>,
) {
    let target_entity = pick_hit.entity();
    if pick_hit.duration.as_millis() >= 100 {
        return;
    }
    // commands.trigger(LogEntityComponents {
    //     entity: target_entity,
    // });

    commands.entity(target_entity).log_components();

    if let Ok(all) = query.get(target_entity) {
        // do something with the components
        bevy::log::info!("{:?}", all);
        bevy::log::info!("intity: {:?}", target_entity.index());
    } else {
        // the entity does not have the components from the query
    }

    bevy::log::info!("hit: {:?}", pick_hit.hit.position);

    match pick_hit.hit.position {
        Some(position) => {
            commands.trigger(CreateNewTagEvent {
                new_tag_dto: NewTagDto {
                    title: "new taaag".to_string(),
                    model_id: 0,
                    position_x: position.x,
                    position_y: position.y,
                    position_z: position.z,
                },
                parent_entity: target_entity,
            });
        }
        None => (),
    };
}

#[derive(Event)]
pub struct RebuildTagsEvent {
    pub new_tag_dtos: Vec<TagDto>,
    pub parent_entity: Entity,
}

fn rebuild_tags(
    trigger: Trigger<RebuildTagsEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query_tags: Query<(Entity, &TagData)>,
) {
    let tags = &trigger.new_tag_dtos;
    let parent_entity = &trigger.parent_entity;
    for old_tag in query_tags.iter() {
        commands.entity(old_tag.0).despawn_recursive();
    }
    bevy::log::info!("tags: {:#?}", tags);
    for tag_dto in tags.iter() {
        commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.0, 0.0),
                    ..Default::default()
                })),
                GlobalTransform::from_xyz(
                    tag_dto.position_x,
                    tag_dto.position_y,
                    tag_dto.position_z,
                ),
                TagData {
                    dto: TagDto {
                        id: tag_dto.id,
                        title: tag_dto.title.clone(),
                        model_id: tag_dto.model_id,
                        created_at: tag_dto.created_at.clone(),
                        position_x: tag_dto.position_x,
                        position_y: tag_dto.position_y,
                        position_z: tag_dto.position_z,
                    },
                },
            ))
            .set_parent_in_place(parent_entity.clone());
    }
}
