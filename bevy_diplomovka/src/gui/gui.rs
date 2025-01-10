use bevy::{ecs::entity, prelude::*};
use bevy_egui::{
    egui::{self, Align2, Id, ScrollArea},
    EguiContexts, EguiPlugin,
};
use bevy_file_dialog::prelude::*;
use dto::{default::TagDto, project::NewProjectDto};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use crate::{
    api::{GetProjectsEvent, UpdateTagEvent},
    building::{ModelData, ProjectData, SelectedTag, TagData, ThisProjectIsSelected},
    utils::compare_by_created_at,
    GameState, ProjectDtoRes,
};
pub struct GuiPlugin;
struct TextFileContents;

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
                    .with_save_file::<TextFileContents>()
                    // allow loading of files marked with TextFileContents
                    .with_load_file::<TextFileContents>(),
            )
            .init_resource::<UiState>()
            .init_resource::<UiContexts>()
            .add_systems(Startup, setup_toasts)
            .add_systems(
                OnEnter(GameState::SelectingProjectAndModel),
                setup_selecting_project_and_model,
            )
            .add_systems(
                Update,
                ui_project_model_screen.run_if(in_state(GameState::SelectingProjectAndModel)),
            )
            .add_systems(
                Update,
                (ui_example_system, ui_tag_windows).run_if(in_state(GameState::ViewingModel)),
            )
            .add_systems(
                Update,
                (
                    file_loaded,
                    file_saved,
                    file_load_canceled,
                    file_save_canceled,
                ),
            )
            .add_observer(show_error);
    }
}

fn setup_selecting_project_and_model(mut commands: Commands) {
    bevy::log::info!("Setting up selecting project and model");
    commands.trigger(GetProjectsEvent {});
}

// this would probably be better id it was in resourse or in single entity
/*
struct ProjectDtoRes {
    pub project_dto: ProjectDto,
}

struct SelectedProjectDtoRes {
    pub project_id: ProjectDto,
}

the queries with ThisProjectIsSelected are adding complexity
*/
fn ui_project_model_screen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    window: Single<&Window>,
    mut query_projects: Query<(Entity, &mut ProjectData, Option<&ThisProjectIsSelected>)>,
    mut modal_open: Local<bool>,
    mut new_project_dto: Local<NewProjectDto>,
) {
    let ctx = contexts.ctx_mut();
    let current_selected_project_entity = query_projects
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(entity, _, _)| entity);

    egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .min_height(32.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("here will be user info");
                });
                // lorem_ipsum(ui);
            });
        });

    egui::SidePanel::left("Projects")
        .resizable(true)
        .default_width(window.width() / 2.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Projects");
            });
            if ui.button("Add new project").clicked() {
                *modal_open = true;
            }
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // lorem_ipsum(ui);
                ui.vertical(|ui| {
                    for (entity, mut project_data, selected_project) in query_projects
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
                        let checked = selected_project.is_some();
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
                            }
                        });

                        ui.separator();
                    }
                })
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Central Panel");
        });
        egui::ScrollArea::vertical().show(ui, |ui| {});
    });

    egui::Window::new("Add new project")
        .open(&mut modal_open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Name: "));
                ui.text_edit_singleline(&mut String::new());
            });
            ui.horizontal(|ui| {
                ui.label(format!("Description: "));
                ui.text_edit_multiline(&mut String::new());
            });
            if ui.button("Submit").clicked() {
                // trigger add new project
            }
        });
}

fn setup_toasts(mut ui_context: ResMut<UiContexts>) {
    ui_context.toasts = Toasts::new()
        .anchor(Align2::RIGHT_TOP, (-10.0, -10.0)) // 10 units from the bottom right corner
        .direction(egui::Direction::TopDown);
}

fn ui_example_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut ui_contexts: ResMut<UiContexts>,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Some things");
            if ui.add(egui::widgets::Button::new("Load model")).clicked() {
                commands
                    .dialog()
                    .add_filter("Text", &["txt"])
                    .load_file::<TextFileContents>();
            }

            if ui.add(egui::widgets::Button::new("Save model")).clicked() {
                commands
                    .dialog()
                    .add_filter("Text", &["txt"])
                    .set_file_name("hello.txt")
                    .save_file::<TextFileContents>(b"hello".to_vec());
            };
            ui.heading("Tags");
            ui.separator();

            ui.vertical(|ui| {
                let scroll_area = ScrollArea::vertical();
                scroll_area.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for (tag_entity, tag, selected_tag) in
                        query_tags.iter().sort_by::<&TagData>(|value_1, value_2| {
                            compare_by_created_at(&value_1.dto.created_at, &value_2.dto.created_at)
                        })
                    {
                        let checked = selected_tag.is_some();
                        if ui
                            .selectable_label(checked, format!("{}: {}", tag.dto.id, tag.dto.title))
                            .clicked()
                        {
                            if checked {
                                commands.entity(tag_entity).remove::<SelectedTag>();
                            } else {
                                commands.entity(tag_entity).insert(SelectedTag {});
                            }
                        }
                    }
                });
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    ui_contexts.toasts.show(ctx);
}

fn ui_tag_windows(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut ui_contexts: ResMut<UiContexts>,
    mut query_tags: Query<(Entity, &mut TagData, &SelectedTag), With<SelectedTag>>,
    query_models: Query<(Entity, &ModelData)>,
) {
    let ctx = contexts.ctx_mut();
    for (entity, mut tag_data, selected_tag) in &mut query_tags {
        egui::Window::new(tag_data.dto.title.clone())
            .id(Id::new(tag_data.dto.id))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Title: "));
                    ui.text_edit_singleline(&mut tag_data.dto.title);
                });
                if ui.button("Submit").clicked() {
                    let parent_entity = query_models
                        .iter()
                        .find(|model| model.1.dto.id == tag_data.dto.model_id);
                    if let Some((target_entity, model_data)) = parent_entity {
                        commands.trigger(UpdateTagEvent {
                            tag_dto: tag_data.dto.clone(),
                            parent_entity: target_entity,
                        });
                    }
                }
                if ui.button("Close").clicked() {
                    commands.entity(entity).remove::<SelectedTag>();
                }
                let scroll_area = ScrollArea::vertical();
                scroll_area.show(ui, |ui| {
                    ui.set_max_height(400.0);
                });
            });
    }
}

#[derive(Event)]
pub struct ShowErrorEvent {
    pub message: String,
}

fn show_error(
    trigger: Trigger<ShowErrorEvent>,
    mut ui_contexts: ResMut<UiContexts>,
    mut contexts: EguiContexts,
) {
    let message = &trigger.event().message;
    let ctx = contexts.ctx_mut();
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

fn file_loaded(mut ev_loaded: EventReader<DialogFileLoaded<TextFileContents>>) {
    for ev in ev_loaded.read() {
        bevy::log::info!(
            "Loaded file {} with contents '{}'",
            ev.file_name,
            std::str::from_utf8(&ev.contents).unwrap()
        );
    }
}

fn file_load_canceled(mut ev_canceled: EventReader<DialogFileLoadCanceled<TextFileContents>>) {
    for _ in ev_canceled.read() {
        bevy::log::info!("Text file content load canceled");
    }
}

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<TextFileContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => bevy::log::info!("File {} successfully saved", ev.file_name),
            Err(ref err) => bevy::log::info!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}

fn file_save_canceled(mut ev_canceled: EventReader<DialogFileSaveCanceled<TextFileContents>>) {
    for _ in ev_canceled.read() {
        bevy::log::info!("Text file content save canceled");
    }
}
