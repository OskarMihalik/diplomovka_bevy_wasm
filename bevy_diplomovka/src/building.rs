use crate::{
    api::{GetModelsEvent, GetTagMessagesEvent, GetUsersInProjectEvent, BACKEND_URL},
    gui::gui::ShowSuccessEvent,
    system_info,
    utils::filter_tags,
};
use bevy::{
    diagnostic::FrameCount, platform::time::Instant, prelude::*,
    render::renderer::RenderAdapterInfo, window::PrimaryWindow,
    world_serialization::WorldInstanceReady,
};
use bevy_mod_outline::*;
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

/// Durations of the steps of opening a model, logged by [`log_model_load_timing`]
#[derive(Component)]
struct ModelLoadTiming {
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
    pub open_time: u128,
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
                (
                    on_tag_filter_change,
                    fade_transparency,
                    set_outline_on_selected,
                    log_model_load_timing,
                )
                    .run_if(in_state(GameState::ViewingModel)),
            )
            .add_observer(rebuild_tags)
            .add_observer(set_outline_on_deselected);
    }
}

pub fn fade_transparency(time: Res<Time>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let alpha = (ops::sin(time.elapsed_secs()) / 2.0) + 0.5;
    for (_, material) in materials.iter_mut() {
        material.base_color.set_alpha(alpha);
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
    query: Query<Entity, (With<ModelData>, With<Transform>, With<WorldAssetRoot>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
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

fn set_outline_on_selected(mut q_tags: Query<(Entity, &mut OutlineVolume), Added<SelectedTag>>) {
    for (_, mut outline) in q_tags.iter_mut() {
        outline.visible = true;
    }
}

fn set_outline_on_deselected(
    trigger: On<Remove, SelectedTag>,
    mut q_tags: Query<(Entity, &mut OutlineVolume)>,
) {
    let entity = trigger.entity;
    if let Ok(mut tag) = q_tags.get_mut(entity) {
        tag.1.visible = false;
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
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
    frame: Res<FrameCount>,
) {
    let selected_model = match selected_model_query {
        Some(ok) => ok,
        None => return,
    };
    bevy::log::info!("model model");
    commands.set_state(GameState::ViewingModel);

    let model_dto = &selected_model.1.dto;
    bevy::log::info!("model {} load: request started", model_dto.id);
    let gltf = asset_server.load(format!(
        "{BACKEND_URL}/assets/model/{:?}.glb#Scene0",
        model_dto.id
    ));
    commands
        .entity(selected_model.0)
        .insert((
            WorldAssetRoot(gltf),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.25)),
            ModelLoadTiming {
                started: Instant::now(),
                started_frame: frame.0,
                last_step: Instant::now(),
                download_parse_ms: None,
                spawn_ms: None,
                spawned_frame: None,
            },
        ))
        .observe(add_tag)
        .observe(
            |trigger: On<WorldInstanceReady>,
             mut commands: Commands,
             q_model: Query<(Entity, &ModelData, &ThisModelIsSelected)>,
             mut q_timing: Query<(&ModelData, &mut ModelLoadTiming)>,
             frame: Res<FrameCount>,
             time: Res<Time>| {
                bevy::log::info!("scene instance ready, {:?}", trigger.entity);
                if let Ok((model, mut timing)) = q_timing.get_mut(trigger.entity) {
                    timing.spawned_frame = Some(frame.0);
                    let spawn_ms = timing.step_ms();
                    timing.spawn_ms = Some(spawn_ms);
                    bevy::log::info!(
                        "model {} load: mesh creation (scene spawn) took {spawn_ms:.1} ms",
                        model.dto.id,
                    );
                }
                let Ok(model) = q_model.get(trigger.entity) else {
                    return;
                };
                bevy::log::info!(
                    "scene rendered in, {:?}",
                    time.elapsed().as_millis() - model.1.open_time,
                );
                commands.trigger(ShowSuccessEvent {
                    message: "Model loaded".to_string(),
                });
            },
        );
}

const LOAD_CSV_HEADER: &str = "date,pc,os,cpu_cores,gpu,graphics_backend,browser,browser_version,build,resolution,model_id,model_name,model_version,meshes,vertices,triangles,download_parse_ms,mesh_creation_ms,first_frame_ms,total_ms,frames";

/// Logs how long downloading + parsing and rendering the first frame took, and a summary of
/// all steps at the end. Steps are noticed once per frame, so they are precise to one frame time.
fn log_model_load_timing(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    frame: Res<FrameCount>,
    mut q_models: Query<(Entity, &ModelData, &WorldAssetRoot, &mut ModelLoadTiming)>,
    q_children: Query<&Children>,
    q_meshes: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    adapter: Res<RenderAdapterInfo>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    for (entity, model, root, mut timing) in &mut q_models {
        let id = model.dto.id;
        if timing.download_parse_ms.is_none() && asset_server.is_loaded_with_dependencies(&root.0) {
            let download_parse_ms = timing.step_ms();
            timing.download_parse_ms = Some(download_parse_ms);
            bevy::log::info!("model {id} load: download + parsing took {download_parse_ms:.1} ms");
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
            let Some(mesh) = meshes.get(mesh) else {
                continue;
            };
            let Ok(positions) = mesh.try_attribute(Mesh::ATTRIBUTE_POSITION) else {
                continue;
            };
            vertices += positions.len();
            triangles += match mesh.try_indices_option() {
                Ok(Some(indices)) => indices.len(),
                _ => positions.len(),
            } / 3;
        }

        let download_parse_ms = timing.download_parse_ms.unwrap_or_default();
        let spawn_ms = timing.spawn_ms.unwrap_or_default();
        let total_ms = timing.total_ms();
        let frames = frame.0.saturating_sub(timing.started_frame);
        let (browser, browser_version) = system_info::browser();
        let resolution = q_window
            .single()
            .map(|window| format!("{}x{}", window.physical_width(), window.physical_height()))
            .unwrap_or_default();
        let csv = system_info::csv_row(&[
            system_info::date(),
            system_info::PC_NAME.to_string(),
            system_info::os(),
            system_info::cpu_cores().to_string(),
            adapter.name.clone(),
            format!("{:?}", adapter.backend),
            browser,
            browser_version,
            system_info::BUILD.to_string(),
            resolution,
            id.to_string(),
            model.dto.name.clone(),
            model.dto.version.to_string(),
            mesh_count.to_string(),
            vertices.to_string(),
            triangles.to_string(),
            format!("{download_parse_ms:.1}"),
            format!("{spawn_ms:.1}"),
            format!("{first_frame_ms:.1}"),
            format!("{total_ms:.1}"),
            frames.to_string(),
        ]);

        bevy::log::info!(
            "\n===== model {id} load: {} (version {}) =====\n\
             download + parsing  {download_parse_ms:>9.1} ms\n\
             mesh creation       {spawn_ms:>9.1} ms\n\
             first frame render  {first_frame_ms:>9.1} ms\n\
             total               {total_ms:>9.1} ms\n\
             meshes              {mesh_count:>9}\n\
             vertices            {vertices:>9}\n\
             triangles           {triangles:>9}\n\
             frames until shown  {frames:>9}\n\
             ----- csv: {LOAD_CSV_HEADER}\n
             {csv}",
            model.dto.name,
            model.dto.version,
        );
        commands.entity(entity).remove::<ModelLoadTiming>();
    }
}

fn add_tag(
    pick_hit: On<Pointer<Click>>,
    mut commands: Commands,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
    query_selected_project: Query<(Entity, &ProjectData, &ThisProjectIsSelected)>,
) {
    let target_entity = pick_hit.entity;
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

    match query_tags.get(pick_hit.original_event_target()) {
        Ok(components) => {
            bevy::log::info!("tag already exists");
            if let Some(_selected) = components.2 {
                commands
                    .entity(pick_hit.original_event_target())
                    .remove::<SelectedTag>();
                return;
            }
            commands
                .entity(pick_hit.original_event_target())
                .insert(SelectedTag {});
            commands.trigger(GetTagMessagesEvent {
                tag_id: components.1.dto.id,
            });
        }
        Err(_) => {
            commands.trigger(CreateNewTagEvent {
                new_tag_dto: NewTagDto {
                    title: "new tag".to_string(),
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
    trigger: On<RebuildTagsEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
) {
    let tags = &trigger.new_tag_dtos;
    let mut selected_tags_id: Vec<i32> = vec![];
    for old_tag in query_tags.iter() {
        commands.entity(old_tag.0).despawn();
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
            OutlineVolume {
                visible: false,
                colour: Color::linear_rgba(1., 1., 0., 1.),
                width: 5.0,
            },
            OutlineMode::FloodFlat,
            Transform::from_xyz(tag_dto.position_x, tag_dto.position_y, tag_dto.position_z)
                .with_scale([tag_dto.scale_x, tag_dto.scale_y, tag_dto.scale_z].into()),
            TagData {
                dto: tag_dto.clone(),
            },
        ));
        builder.observe(add_tag);

        if selected_tags_id.contains(&tag_dto.id) {
            builder.insert(SelectedTag {});
            commands.trigger(GetTagMessagesEvent { tag_id: tag_dto.id });
        }
    }
}
