use crate::{
    api::{GetModelsEvent, GetTagMessagesEvent, GetUsersInProjectEvent, BACKEND_URL},
    utils::filter_tags,
};
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use dto::{
    default::{NewTagDto, StatusDto, TagDto, TagMessageDto},
    model::ModelDto,
    project::ProjectDto,
};

use crate::{
    api::{CreateNewTagEvent, GetTagsEvent},
    GameState,
};

#[derive(Component)]
pub struct TagData {
    pub dto: TagDto,
}

#[derive(Component)]
pub struct TagMessagesData {
    pub dtos: Vec<TagMessageDto>,
}

#[derive(Component)]
pub struct SelectedTag {}

#[derive(Component)]
pub struct TagHasOpenStatusModal {}

#[derive(Component)]
pub struct ModelData {
    pub dto: ModelDto,
}
#[derive(Component)]
pub struct ThisModelIsSelected {}

#[derive(Component)]
pub struct ProjectData {
    pub dto: ProjectDto,
}

#[derive(Component)]

pub struct ThisProjectIsSelected {}

#[derive(Component)]

pub struct ProjectStatusesData {
    pub dtos: Vec<StatusDto>,
}
#[derive(Component, Clone)]

pub struct TagFilter {
    pub title: String,
    pub status_title: String,
}

#[derive(Component)]
pub struct KanbanOpen {}

#[derive(Component)]
pub struct LightControlsOpen {}

pub struct BuildingPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene)
            .add_systems(OnEnter(GameState::ViewingModel), on_viewing_model)
            .add_systems(OnExit(GameState::ViewingModel), on_exit_viewing_model)
            .add_systems(
                Update,
                react_to_project_change.run_if(in_state(GameState::SelectingProjectAndModel)),
            )
            .add_systems(
                Update,
                react_to_model_change.run_if(in_state(GameState::SelectingProjectAndModel)),
            )
            .add_systems(
                Update,
                on_tag_filter_change.run_if(in_state(GameState::ViewingModel)),
            )
            .add_observer(rebuild_tags)
            .add_observer(spawn_building);
    }
}

fn on_tag_filter_change(
    query_tag_filter: Query<&TagFilter, Changed<TagFilter>>,
    mut query_tags: Query<(&TagData, &mut Visibility)>,
) {
    let filter = match query_tag_filter.iter().next() {
        Some(filter) => filter,
        None => return,
    };

    for (tag_data, mut visibility) in query_tags.iter_mut() {
        if filter_tags(
            &tag_data.dto.title,
            &tag_data
                .dto
                .status_dto
                .as_ref()
                .map_or("".to_string(), |s| s.title.clone()),
            &filter.title,
            &filter.status_title,
        ) {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn on_exit_viewing_model(
    mut commands: Commands,
    query: Query<Entity, (With<ModelData>, With<Transform>, With<SceneRoot>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn on_viewing_model(
    mut commands: Commands,
    query: Query<&ProjectData, (Changed<ProjectData>, With<ThisProjectIsSelected>)>,
) {
    for project_data in query.iter() {
        commands.trigger(GetTagsEvent {
            project_id: project_data.dto.id,
        });
    }
}
#[derive(Event)]
pub struct SpawnModelEvent {
    pub model_dto: ModelDto,
}

fn spawn_building(
    trigger: Trigger<SpawnModelEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let model_dto = &trigger.model_dto;
    let gltf = asset_server.load(format!(
        "{BACKEND_URL}/assets/model/{:?}.glb#Scene0",
        model_dto.id
    ));
    commands
        .spawn((
            SceneRoot(gltf),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.25)),
            // ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            // PickableBundle::default(),
            // AvianPickable,
            // RigidBody::Static,
            ModelData {
                dto: model_dto.clone(),
            },
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
        PanOrbitCamera {
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Middle,
            ..Default::default()
        },
    ));

    commands.spawn(TagFilter {
        title: "".to_string(),
        status_title: "".to_string(),
    });
}

fn react_to_project_change(
    mut commands: Commands,
    selected_project_query: Option<
        Single<(&ProjectData, &ThisProjectIsSelected), Added<ThisProjectIsSelected>>,
    >,
) {
    let selected_project = match selected_project_query {
        Some(ok) => ok,
        None => return,
    };
    bevy::log::info!("project changed");
    commands.trigger(GetModelsEvent {
        project_id: selected_project.0.dto.id,
    });
    commands.trigger(GetUsersInProjectEvent {
        project_id: selected_project.0.dto.id,
    });
}

fn react_to_model_change(
    mut commands: Commands,
    selected_model_query: Option<
        Single<(Entity, &ModelData, &ThisModelIsSelected), Added<ThisModelIsSelected>>,
    >,
    asset_server: Res<AssetServer>,
) {
    let selected_model = match selected_model_query {
        Some(ok) => ok,
        None => return,
    };
    bevy::log::info!("model model");
    commands.set_state(GameState::ViewingModel);

    let model_dto = &selected_model.1.dto;
    let gltf = asset_server.load(format!(
        "{BACKEND_URL}/assets/model/{:?}.glb#Scene0",
        model_dto.id
    ));
    commands
        .entity(selected_model.0)
        .insert((
            SceneRoot(gltf),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.25)),
            // ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            // PickableBundle::default(),
            // AvianPickable,
            // RigidBody::Static,
            // ModelData {
            //     dto: model_dto.clone(),
            // },
        ))
        .observe(add_tag);
}

fn add_tag(
    pick_hit: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
    query_selected_project: Query<(Entity, &ProjectData, &ThisProjectIsSelected)>,
) {
    let target_entity = pick_hit.entity();
    if pick_hit.duration.as_millis() >= 100 {
        return;
    }

    let Some((_, project_data, _)) = query_selected_project.iter().next() else {
        return;
    };

    commands.entity(target_entity).log_components();

    bevy::log::info!("hit: {:?}", pick_hit.hit.position);

    let position = match pick_hit.hit.position {
        Some(position) => position,
        None => return,
    };

    match query_tags.get(pick_hit.target) {
        Ok(components) => {
            bevy::log::info!("tag already exists");
            if let Some(_selected) = components.2 {
                commands.entity(pick_hit.target).remove::<SelectedTag>();
                return;
            }
            commands.entity(pick_hit.target).insert(SelectedTag {});
            commands.trigger(GetTagMessagesEvent {
                tag_id: components.1.dto.id,
            });
        }
        Err(_) => {
            commands.trigger(CreateNewTagEvent {
                new_tag_dto: NewTagDto {
                    title: "new taaag".to_string(),
                    project_id: project_data.dto.id,
                    position_x: position.x,
                    position_y: position.y,
                    position_z: position.z,
                },
                parent_entity: target_entity,
            });
        }
    }
}

#[derive(Event)]
pub struct RebuildTagsEvent {
    pub new_tag_dtos: Vec<TagDto>,
}

fn rebuild_tags(
    trigger: Trigger<RebuildTagsEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
) {
    let tags = &trigger.new_tag_dtos;
    let mut selected_tags_id: Vec<i32> = vec![];
    for old_tag in query_tags.iter() {
        commands.entity(old_tag.0).despawn_recursive();
        if old_tag.2.is_some() {
            selected_tags_id.push(old_tag.1.dto.id);
        }
    }
    bevy::log::info!("tags: {:#?}", tags);
    for tag_dto in tags.iter() {
        let mesh_handle =
            tag_dto
                .status_dto
                .clone()
                .map_or(meshes.add(Cuboid::default()), |status| {
                    return match status.shape {
                        dto::default::Shape::Cuboid => meshes.add(Cuboid::default()),
                        dto::default::Shape::Tetrahedron => meshes.add(Tetrahedron::default()),
                        dto::default::Shape::Capsule3d => meshes.add(Capsule3d::default()),
                        dto::default::Shape::Torus => meshes.add(Torus::default()),
                        dto::default::Shape::Cylinder => meshes.add(Cylinder::default()),
                        dto::default::Shape::Cone => meshes.add(Cone::default()),
                        dto::default::Shape::ConicalFrustum => {
                            meshes.add(ConicalFrustum::default())
                        }
                        dto::default::Shape::Sphere => meshes.add(Sphere::default()),
                    };
                });
        let mut builder = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(
                materials.add(StandardMaterial {
                    base_color: Color::srgb(
                        tag_dto
                            .status_dto
                            .clone()
                            .map_or(1.0, |status| status.color_r),
                        tag_dto
                            .status_dto
                            .clone()
                            .map_or(0.0, |status| status.color_g),
                        tag_dto
                            .status_dto
                            .clone()
                            .map_or(0.0, |status| status.color_b),
                    ),
                    ..Default::default()
                }),
            ),
            Transform::from_xyz(tag_dto.position_x, tag_dto.position_y, tag_dto.position_z)
                .with_scale([tag_dto.scale_x, tag_dto.scale_y, tag_dto.scale_z].into()),
            TagData {
                dto: tag_dto.clone(),
            },
        ));
        builder.observe(add_tag);

        if selected_tags_id.contains(&tag_dto.id) {
            builder.insert(SelectedTag {});
            builder.trigger(GetTagMessagesEvent { tag_id: tag_dto.id });
        }
    }
}
