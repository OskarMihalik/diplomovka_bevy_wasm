use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Id},
    EguiContexts, EguiPlugin,
};
use bevy_file_dialog::prelude::*;
use dto::{
    model::UpdateModelDto,
    project::ProjectDto,
    users::{GetUsersDto, OtherUserDto, ProjectUserDto, UserToProjectDto},
};
use std::collections::HashMap;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use crate::{
    api::{
        AddUserToProjectEvent, CreateModelEvent, DeleteProjectEvent, GetProjectsEvent,
        GetUsersEvent, NewProjectEvent, UpdateModelEvent, UpdateProjectEvent,
    },
    api_tracking::ApiStatus,
    building::{ModelData, ProjectData, ThisModelIsSelected, ThisProjectIsSelected},
    users::{LoggedUser, OtherUsers, UsersInProject},
    utils::compare_by_created_at,
    GameState,
};

use super::{
    auth_screen::{login_screen, reset_auth_forms, AuthForms},
    confirm_modal::{confirm_modal, ConfirmModalResult},
    glb_picker::{set_uploaded_glb, GlbPickerPlugin},
    kanban::kanban_window,
    new_model_modal::{new_model_modal, NewModelModal},
    new_project_modal::{new_project_modal, NewProjectModal},
    light_controls::{light_controls_window, on_light_gizmos_added, on_light_gizmos_removed},
    status_modal::ui_status_modal,
    theme::theme_picker,
    viewing_model::{ui_left_panel, ui_tag_windows, update_filter_change},
};
pub struct GuiPlugin;
pub struct GlbFileContents;

#[derive(Component)]
pub struct UploadedGlbFile {
    pub file_name: String,
    pub contents: Vec<u8>,
}

#[derive(Default, Resource)]
pub struct UiState {}

#[derive(Default, Resource)]
pub struct UiContexts {
    pub toasts: Toasts,
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(GlbPickerPlugin)
            .add_plugins(
                FileDialogPlugin::new()
                    // allow saving of files marked with TextFileContents
                    .with_save_file::<GlbFileContents>()
                    // allow loading of files marked with TextFileContents
                    .with_load_file::<GlbFileContents>(),
            )
            .init_resource::<UiState>()
            .init_resource::<UiContexts>()
            .init_resource::<AuthForms>()
            .add_systems(Startup, setup_toasts)
            .add_systems(
                OnEnter(GameState::SelectingProjectAndModel),
                setup_selecting_project_and_model,
            )
            .add_systems(Update, (login_screen).run_if(in_state(GameState::Auth)))
            .add_systems(OnExit(GameState::Auth), reset_auth_forms)
            .add_systems(
                Update,
                (ui_project_screen, ui_model_screen)
                    .chain()
                    .run_if(in_state(GameState::SelectingProjectAndModel)),
            )
            .add_systems(
                Update,
                (
                    ui_left_panel,
                    ui_tag_windows,
                    ui_status_modal,
                    kanban_window,
                    light_controls_window,
                    on_light_gizmos_added,
                    on_light_gizmos_removed,
                )
                    .run_if(in_state(GameState::ViewingModel)),
            )
            .add_systems(
                Update,
                (
                    theme_picker,
                    file_loaded,
                    file_saved,
                    file_load_canceled,
                    file_save_canceled,
                    show_toasts,
                ),
            )
            .add_observer(update_filter_change)
            .add_observer(show_success)
            .add_observer(show_error);
    }
}

fn setup_selecting_project_and_model(mut commands: Commands) {
    bevy::log::info!("Setting up selecting project and model");
    commands.trigger(GetProjectsEvent {});
}

// this would probably be better id it was in resourse or in single entity
/*
struct Projects {
    pub project_dto: Vec<ProjectDto>,
}

struct SelectedProject {
    pub project_id: i32,
}

the queries with ThisProjectIsSelected are adding complexity but it works so whatever
*/
fn ui_model_screen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    query_projects: Query<(Entity, &ProjectData, Option<&ThisProjectIsSelected>)>,
    query_models: Query<(Entity, &ModelData, Option<&ThisModelIsSelected>)>,
    mut model_name_drafts: Local<HashMap<i32, String>>,
    mut new_model: Local<NewModelModal>,
    mut new_model_status: ApiStatus<CreateModelEvent>,
    query_loaded_glb: Query<(Entity, &UploadedGlbFile)>,
) {
    let ctx = contexts.ctx_mut();
    let current_selected_project = query_projects
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(entity, project_data, _)| (entity, project_data));

    let (_, current_selected_project_dto) = match current_selected_project {
        Some((entity, project_data)) => (Some(entity), Some(project_data)),
        None => (None, None),
    };

    let current_selected_model_entity = query_models
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(entity, _, _)| entity);

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Models");
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // lorem_ipsum(ui);
            ui.vertical(|ui| {
                if current_selected_project_dto.is_some() && ui.button("Add new model").clicked() {
                    new_model.open = true;
                }
                let mut v = 0;
                for (entity, model_data, selected_model) in query_models
                    .iter()
                    .sort_by::<&ModelData>(|value_1, value_2| {
                        compare_by_created_at(&value_1.dto.created_at, &value_2.dto.created_at)
                    })
                {
                    // edits go into a draft, ModelData stays what the server sent
                    let saved_name = &model_data.dto.name;
                    let mut draft_name = model_name_drafts
                        .get(&model_data.dto.id)
                        .cloned()
                        .unwrap_or_else(|| saved_name.clone());

                    egui::Grid::new(Id::new(model_data.dto.id))
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(format!("Name: "));
                            ui.text_edit_singleline(&mut draft_name);
                            ui.end_row();

                            ui.label(format!("Version: "));
                            ui.label(&v.to_string());
                            ui.end_row();
                            v += 1;

                            ui.label(format!("Id: "));
                            ui.label(model_data.dto.id.to_string());
                            ui.end_row();
                        });
                    let checked = selected_model.is_some();
                    ui.horizontal(|ui| {
                        if ui.selectable_label(checked, format!("Open")).clicked() {
                            // if checked {
                            bevy::log::info!("Open model clicked");
                            match current_selected_model_entity {
                                Some(selected) => {
                                    commands.entity(selected).remove::<ThisModelIsSelected>();
                                    commands.entity(entity).insert(ThisModelIsSelected {});
                                }
                                None => {
                                    commands.entity(entity).insert(ThisModelIsSelected {});
                                }
                            }
                        }
                        // only offer saving when the name was edited
                        if draft_name != *saved_name {
                            let valid = !draft_name.trim().is_empty();
                            if ui
                                .add_enabled(valid, egui::Button::new("Submit"))
                                .on_disabled_hover_text("Name must not be empty")
                                .clicked()
                            {
                                commands.trigger(UpdateModelEvent {
                                    dto: UpdateModelDto {
                                        id: model_data.dto.id,
                                        name: draft_name.trim().to_string(),
                                    },
                                });
                            }
                            if ui.button("Revert").clicked() {
                                draft_name = saved_name.clone();
                            }
                        }
                    });

                    // a draft equal to the server data (saved or reverted) isn't needed anymore
                    if draft_name == *saved_name {
                        model_name_drafts.remove(&model_data.dto.id);
                    } else {
                        model_name_drafts.insert(model_data.dto.id, draft_name);
                    }

                    ui.separator();
                }
            })
        });
    });
    new_model_modal(
        ctx,
        &mut commands,
        &mut new_model,
        &mut new_model_status,
        current_selected_project_dto.map(|project| project.dto.id),
        query_loaded_glb.get_single().ok(),
    );
}

/// Unsaved edits of a project in the project list.
#[derive(Clone, PartialEq)]
struct ProjectDraft {
    name: String,
    description: String,
}

impl ProjectDraft {
    fn from_dto(dto: &ProjectDto) -> Self {
        Self {
            name: dto.name.clone(),
            description: dto.description.clone(),
        }
    }
}

fn ui_project_screen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    window: Single<&Window>,
    query_projects: Query<(Entity, &ProjectData, Option<&ThisProjectIsSelected>)>,
    mut project_drafts: Local<HashMap<i32, ProjectDraft>>,
    mut new_project: Local<NewProjectModal>,
    mut modal_add_user_open: Local<bool>,
    mut new_project_status: ApiStatus<NewProjectEvent>,
    query_project_users: Option<Single<(Entity, &UsersInProject)>>,
    mut user_email: Local<String>,
    mut open_delete_dialog: Local<bool>,
    mut project_to_delete: Local<i32>,
    other_users: Option<Single<(Entity, &OtherUsers)>>,
    query_logged_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let ctx = contexts.ctx_mut();
    let logged_user = match query_logged_user {
        Some(some) => some.1,
        None => {
            egui::Modal::new(Id::new("No logged in user")).show(ctx, |ui| {
                ui.label("No logged user");
                if ui.button("Go back to login").clicked() {
                    commands.set_state(GameState::Auth);
                }
            });
            return;
        }
    };
    let current_selected_project_entity = query_projects
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(entity, _, _)| (entity));

    let project_dto = query_projects
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(_, project_data, _)| (project_data.dto.clone()));

    let project_users = match query_project_users {
        Some(some) => &some.1.dtos,
        None => &Vec::<ProjectUserDto>::new(),
    };

    egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .min_height(32.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if project_dto.is_some() && ui.button("Add user").clicked() {
                    *modal_add_user_open = true;
                    commands.trigger(GetUsersEvent {
                        dto: GetUsersDto {
                            email: "".to_string(),
                        },
                    });
                }
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for user in project_users {
                        ui.horizontal(|ui| {
                            ui.label(user.email.clone());
                            if let Some(dto) = &project_dto {
                                if user.is_admin {
                                    ui.label("Admin");
                                } else {
                                    ui.label("User");
                                }
                            }
                            ui.add(egui::Separator::default().vertical());
                        });
                    }
                })
            })
        });

    egui::SidePanel::left("Projects")
        .resizable(true)
        .default_width(window.width() / 2.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Projects");
            });
            if ui.button("Log out").clicked() {
                commands.set_state(GameState::Auth);
            }
            ui.horizontal(|ui| {
                if ui.button("Add new project").clicked() {
                    new_project.open = true;
                }
                if ui.button("⟲").clicked() {
                    commands.trigger(GetProjectsEvent {});
                };
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // lorem_ipsum(ui);
                ui.vertical(|ui| {
                    for (entity, project_data, selected_model) in query_projects
                        .iter()
                        .sort_by::<&ProjectData>(
                        |value_1, value_2| {
                            compare_by_created_at(&value_1.dto.created_at, &value_2.dto.created_at)
                        },
                    ) {
                        // edits go into a draft, ProjectData stays what the server sent
                        let saved = ProjectDraft::from_dto(&project_data.dto);
                        let mut draft = project_drafts
                            .get(&project_data.dto.id)
                            .cloned()
                            .unwrap_or_else(|| saved.clone());

                        egui::Grid::new(Id::new(project_data.dto.id))
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(format!("Name: "));
                                ui.text_edit_singleline(&mut draft.name);
                                ui.end_row();

                                ui.label(format!("Description: "));
                                ui.text_edit_multiline(&mut draft.description);
                                ui.end_row();
                            });
                        let checked = selected_model.is_some();
                        ui.horizontal(|ui| {
                            if ui.selectable_label(checked, format!("Select")).clicked() {
                                // if checked {
                                match current_selected_project_entity {
                                    Some(selected) => {
                                        commands.entity(selected).remove::<ThisProjectIsSelected>();
                                        commands.entity(entity).insert(ThisProjectIsSelected {});
                                    }
                                    None => {
                                        commands.entity(entity).insert(ThisProjectIsSelected {});
                                    }
                                }
                            }
                            // only offer saving when something was edited
                            if draft != saved {
                                if ui.button("Submit").clicked() {
                                    commands.trigger(UpdateProjectEvent {
                                        dto: ProjectDto {
                                            name: draft.name.clone(),
                                            description: draft.description.clone(),
                                            ..project_data.dto.clone()
                                        },
                                    });
                                }
                                if ui.button("Revert").clicked() {
                                    draft = saved.clone();
                                }
                            }
                            if logged_user.dto.id == project_data.dto.created_by_id
                                && ui.button("Delete").clicked()
                            {
                                *open_delete_dialog = true;
                                *project_to_delete = project_data.dto.id;
                            }
                        });

                        // a draft equal to the server data (saved or reverted) isn't needed anymore
                        if draft == saved {
                            project_drafts.remove(&project_data.dto.id);
                        } else {
                            project_drafts.insert(project_data.dto.id, draft);
                        }

                        ui.separator();
                    }
                })
            });
        });

    new_project_modal(ctx, &mut commands, &mut new_project, &mut new_project_status);

    match confirm_modal(ctx, "Delete project?", &open_delete_dialog, 123) {
        ConfirmModalResult::Confirm => {
            *open_delete_dialog = false;
            commands.trigger(DeleteProjectEvent {
                project_id: *project_to_delete,
            });
        }
        ConfirmModalResult::Cancel => {
            *open_delete_dialog = false;
        }
        ConfirmModalResult::Nothing => (),
    }

    if *modal_add_user_open {
        let modal = egui::Modal::new(Id::new("Add user to project modal")).show(ctx, |ui| {
            ui.label("User email:");
            ui.text_edit_singleline(&mut *user_email);
            if ui.button("Search").clicked() {
                commands.trigger(GetUsersEvent {
                    dto: GetUsersDto {
                        email: user_email.clone(),
                    },
                });
            }
            let other_users = match other_users {
                Some(some) => &some.1.dtos,
                None => &Vec::<OtherUserDto>::new(),
            };
            match project_dto {
                Some(some_project_dto) => {
                    for other_user in other_users {
                        ui.horizontal(|ui| {
                            ui.label(other_user.username.clone());
                            ui.label(other_user.email.clone());
                            if ui.button("Add").clicked() {
                                commands.trigger(AddUserToProjectEvent {
                                    dto: UserToProjectDto {
                                        project_id: some_project_dto.id,
                                        user_id: other_user.id,
                                    },
                                });
                            }
                        });
                    }
                    ui.separator();
                    if other_users.is_empty() {
                        ui.label("No users found");
                    }
                }
                None => (),
            };
        });
        if modal.should_close() {
            *modal_add_user_open = false;
        }
    }
}

fn setup_toasts(mut ui_context: ResMut<UiContexts>) {
    ui_context.toasts = Toasts::new()
        .anchor(Align2::RIGHT_TOP, (-10.0, -10.0)) // 10 units from the bottom right corner
        .direction(egui::Direction::TopDown);
}

#[derive(Event)]
pub struct ShowErrorEvent {
    pub message: String,
}

fn show_error(trigger: Trigger<ShowErrorEvent>, mut ui_contexts: ResMut<UiContexts>) {
    let message = &trigger.event().message;
    bevy::log::error!("Error: {}", message);
    ui_contexts.toasts.add(Toast {
        text: message.into(),
        kind: ToastKind::Error,
        options: ToastOptions::default()
            .duration_in_seconds(5.0)
            .show_progress(true),
        ..Default::default()
    });
}

#[derive(Event)]
pub struct ShowSuccessEvent {
    pub message: String,
}

fn show_success(trigger: Trigger<ShowSuccessEvent>, mut ui_contexts: ResMut<UiContexts>) {
    let message = &trigger.event().message;
    ui_contexts.toasts.add(Toast {
        text: message.into(),
        kind: ToastKind::Success,
        options: ToastOptions::default()
            .duration_in_seconds(5.0)
            .show_progress(true),
        ..Default::default()
    });
}

fn show_toasts(mut contexts: EguiContexts, mut ui_contexts: ResMut<UiContexts>) {
    let ctx = contexts.ctx_mut();
    ui_contexts.toasts.show(ctx);
}

fn file_loaded(
    mut ev_loaded: EventReader<DialogFileLoaded<GlbFileContents>>,
    mut commands: Commands,
    query_loaded_glb: Query<Entity, With<UploadedGlbFile>>,
) {
    for ev in ev_loaded.read() {
        set_uploaded_glb(
            &mut commands,
            query_loaded_glb.iter(),
            ev.file_name.clone(),
            ev.contents.clone(),
        );
    }
}

fn file_load_canceled(mut ev_canceled: EventReader<DialogFileLoadCanceled<GlbFileContents>>) {
    for _ in ev_canceled.read() {
        bevy::log::info!("Text file content load canceled");
    }
}

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<GlbFileContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => bevy::log::info!("File {} successfully saved", ev.file_name),
            Err(ref err) => bevy::log::info!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}

fn file_save_canceled(mut ev_canceled: EventReader<DialogFileSaveCanceled<GlbFileContents>>) {
    for _ in ev_canceled.read() {
        bevy::log::info!("Text file content save canceled");
    }
}
