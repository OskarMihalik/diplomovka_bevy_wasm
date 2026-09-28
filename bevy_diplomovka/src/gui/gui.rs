use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Id},
    EguiContexts, EguiPlugin,
};
use bevy_file_dialog::prelude::*;
use dto::{
    model::NewModelDto,
    users::{GetUsersDto, OtherUserDto, ProjectUserDto, UserToProjectDto},
};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use crate::{
    api::{
        AddUserToProjectEvent, CreateModelEvent, DeleteProjectEvent, GetProjectsEvent,
        GetUsersEvent, NewProjectEvent, UpdateProjectEvent,
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
    kanban::kanban_window,
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
    mut query_models: Query<(Entity, &mut ModelData, Option<&ThisModelIsSelected>)>,
    mut new_model_modal_open: Local<bool>,
    mut new_model_dto: Local<NewModelDto>,
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
                    *new_model_modal_open = true;
                }
                let mut v = 0;
                for (entity, mut model_data, selected_model) in query_models
                    .iter_mut()
                    .sort_by::<&ModelData>(|value_1, value_2| {
                        compare_by_created_at(&value_1.dto.created_at, &value_2.dto.created_at)
                    })
                {
                    egui::Grid::new(Id::new(model_data.dto.id))
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            // self.gallery_grid_contents(ui);
                            ui.label(format!("Name: "));
                            ui.text_edit_singleline(&mut model_data.dto.name);
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
                        if ui.button("Submit").clicked() {
                            // trigger update project
                        }
                    });

                    ui.separator();
                }
            })
        });
    });
    if *new_model_modal_open {
        let modal = egui::Modal::new(Id::new("Add new model modal")).show(ctx, |ui| {
            ui.label("Model name:");
            ui.text_edit_singleline(&mut new_model_dto.name);
            match query_loaded_glb.get_single() {
                Ok((entity, uploaded_glb)) => {
                    ui.label("Glb file loaded");
                    ui.label(&uploaded_glb.file_name);
                    if ui.button("Submit").clicked() {
                        match current_selected_project_dto {
                            Some(selected_project) => {
                                commands.trigger(CreateModelEvent {
                                    name: new_model_dto.name.clone(),
                                    project_id: selected_project.dto.id,
                                    model_bytes: uploaded_glb.contents.clone(),
                                });
                                commands.entity(entity).despawn_recursive();
                            }
                            None => {
                                bevy::log::error!("No project selected");
                            }
                        }
                        // commands.trigger(CreateModelEvent {
                        //     name: new_model_dto.name.clone(),
                        //     project_id: 0,
                        //     model_bytes: uploaded_glb.contents.clone(),
                        // });
                        // commands.entity(entity).despawn_recursive();
                    }
                }
                Err(_) => {
                    ui.label("No glb file loaded");
                    if ui.button("Load glb file").clicked() {
                        commands
                            .dialog()
                            .add_filter("Glb", &["glb"])
                            .load_file::<GlbFileContents>();
                    }
                }
            }
        });
        if modal.should_close() {
            *new_model_modal_open = false;
            new_model_dto.name = String::new();
            new_model_dto.project_id = 0;
        }
    }
}

fn ui_project_screen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    window: Single<&Window>,
    mut query_projects: Query<(Entity, &mut ProjectData, Option<&ThisProjectIsSelected>)>,
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
                    for (entity, mut project_data, selected_model) in query_projects
                        .iter_mut()
                        .sort_by::<&ProjectData>(
                        |value_1, value_2| {
                            compare_by_created_at(&value_1.dto.created_at, &value_2.dto.created_at)
                        },
                    ) {
                        egui::Grid::new(Id::new(project_data.dto.id))
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                // self.gallery_grid_contents(ui);
                                ui.label(format!("Name: "));
                                ui.text_edit_singleline(&mut project_data.dto.name);
                                ui.end_row();

                                ui.label(format!("Description: "));
                                ui.text_edit_multiline(&mut project_data.dto.description);
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
                            if ui.button("Submit").clicked() {
                                // trigger update project
                                commands.trigger(UpdateProjectEvent {
                                    dto: project_data.dto.clone(),
                                });
                            }
                            if logged_user.dto.id == project_data.dto.created_by_id
                                && ui.button("Delete").clicked()
                            {
                                *open_delete_dialog = true;
                                *project_to_delete = project_data.dto.id;
                            }
                        });

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
) {
    for ev in ev_loaded.read() {
        bevy::log::info!("Loaded file {} with contents", ev.file_name);
        commands.spawn(UploadedGlbFile {
            file_name: ev.file_name.clone(),
            contents: ev.contents.clone(),
        });
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
