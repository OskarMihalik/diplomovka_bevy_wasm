use bevy::prelude::*;
use bevy_egui::{
    egui::{
        self,
        text::{LayoutJob, TextWrapping},
        CornerRadius, Id, ScrollArea, TextFormat, TextStyle,
    },
    EguiContexts,
};
use dto::default::{NewTagMessageDto, StatusDto, TagDto};
use std::collections::HashMap;

use crate::{
    api::{
        CreateTagMessageEvent, DeleteTagEvent, GetStatusesEvent, GetTagMessagesEvent, GetTagsEvent,
        UpdateTagEvent,
    },
    building::{
        KanbanOpen, LightControlsOpen, ModelData, ProjectData, SelectedTag, TagData, TagFilter,
        TagHasOpenStatusModal, TagMessagesData, ThisProjectIsSelected,
    },
    utils::{compare_by_created_at, convert_color_to_egui, filter_tags},
    GameState,
};
use egui_commonmark::*;

use super::confirm_modal::{confirm_modal, ConfirmModalResult};
#[derive(Event)]
pub struct FilterChangeEvent {
    pub new_filter: TagFilter,
}
pub fn update_filter_change(
    trigger: On<FilterChangeEvent>,
    mut tag_filter: Single<&mut TagFilter>,
) {
    **tag_filter = trigger.new_filter.clone();
}

fn filter_combobox_text(title: &str) -> String {
    if title.is_empty() {
        "All".to_string()
    } else {
        title.to_string()
    }
}

pub fn ui_left_panel(
    mut commands: Commands,
    mut contexts: EguiContexts,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
    mut tag_filter: Single<&mut TagFilter>,
    q_kanban_open: Query<(Entity, &KanbanOpen)>,
    q_project: Single<(&ProjectData, &ThisProjectIsSelected)>,
    q_light_controls: Query<(Entity, &LightControlsOpen)>,
    q_projects: Query<&ProjectData, With<ThisProjectIsSelected>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let kanban = q_kanban_open.iter().next();
    let light_controls = q_light_controls.iter().next();
    let Some(project_data) = q_projects.iter().next() else {
        return;
    };
    egui::Panel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            if ui.button("⬅").clicked() {
                commands.set_state(GameState::SelectingProjectAndModel);
            }
            let inner_tag_filter = tag_filter.bypass_change_detection();
            egui::Grid::new(Id::new("Tags filter"))
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    if ui.selectable_label(kanban.is_some(), "Kanban").clicked() {
                        match kanban {
                            Some((entity, _)) => {
                                commands.entity(entity).despawn();
                            }
                            None => {
                                commands.spawn(KanbanOpen {});
                                commands.trigger(GetStatusesEvent {
                                    project_id: q_project.0.dto.id,
                                });
                            }
                        }
                    }

                    if ui
                        .selectable_label(light_controls.is_some(), "Lights")
                        .clicked()
                    {
                        match light_controls {
                            Some((entity, _)) => {
                                commands.entity(entity).despawn();
                            }
                            None => {
                                commands.spawn(LightControlsOpen {});
                            }
                        }
                    }
                    ui.end_row();

                    ui.heading("Tags");
                    ui.text_edit_singleline(&mut inner_tag_filter.title);
                    ui.end_row();

                    ui.heading("Status:");
                    egui::ComboBox::new(Id::new("Select status"), "")
                        .selected_text(filter_combobox_text(&inner_tag_filter.status_title))
                        .show_ui(ui, |ui| {
                            let mut unique_statuses = query_tags
                                .iter()
                                .filter_map(|(_, tag, _)| tag.dto.status_dto.clone())
                                .collect::<Vec<StatusDto>>();

                            unique_statuses.sort_by(|a, b| a.title.cmp(&b.title));
                            unique_statuses.dedup_by(|a, b| a.title == b.title);

                            for status in unique_statuses {
                                ui.selectable_value(
                                    &mut inner_tag_filter.status_title,
                                    status.title.clone(),
                                    status.title.clone(),
                                );
                            }
                            ui.selectable_value(
                                &mut inner_tag_filter.status_title,
                                "".to_string(),
                                "All",
                            );
                        });

                    ui.end_row();
                    if ui.button("Apply").clicked() {
                        commands.trigger(FilterChangeEvent {
                            new_filter: inner_tag_filter.clone(),
                        });
                    }
                    if ui.button("Reset").clicked() {
                        inner_tag_filter.title.clear();
                        inner_tag_filter.status_title.clear();
                        commands.trigger(FilterChangeEvent {
                            new_filter: inner_tag_filter.clone(),
                        });
                    }
                });
            ui.separator();

            ui.vertical(|ui| {
                if ui.button("⟲").clicked() {
                    commands.trigger(GetTagsEvent {
                        project_id: project_data.dto.id,
                    });
                };
                let scroll_area = ScrollArea::vertical();
                scroll_area.show(ui, |ui| {
                    egui::Grid::new(Id::new("Tags filter"))
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .max_col_width(100.0)
                        .show(ui, |ui| {
                            for (tag_entity, tag, selected_tag) in query_tags
                                .iter()
                                .sort_by::<&TagData>(|value_1, value_2| {
                                    compare_by_created_at(
                                        &value_1.dto.created_at,
                                        &value_2.dto.created_at,
                                    )
                                })
                                .filter(|(_, tag, _)| {
                                    filter_tags(
                                        &tag.dto.title,
                                        &tag.dto
                                            .status_dto
                                            .as_ref()
                                            .map_or("".to_string(), |s| s.title.clone()),
                                        &inner_tag_filter.title,
                                        &inner_tag_filter.status_title,
                                    )
                                })
                            {
                                let checked = selected_tag.is_some();
                                let text = format!("{}: {}", tag.dto.id, tag.dto.title);
                                let mut job = LayoutJob::default();
                                let format = TextFormat {
                                    font_id: TextStyle::Button.resolve(ui.style()),
                                    ..Default::default()
                                };
                                job.append(text.as_str(), 0.0, format);
                                job.wrap = TextWrapping {
                                    max_rows: 1,
                                    break_anywhere: true,
                                    ..Default::default()
                                };

                                if ui.selectable_label(checked, job).clicked() {
                                    if checked {
                                        commands.entity(tag_entity).remove::<SelectedTag>();
                                    } else {
                                        commands.entity(tag_entity).insert(SelectedTag {});
                                    }
                                }
                                match &tag.dto.status_dto {
                                    Some(status_dto) => {
                                        status_widget(ui, status_dto);
                                    }
                                    None => {
                                        ui.label("No status");
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
            });
            // ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });
}

pub fn ui_tag_windows(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut query_tags: Query<
        (
            Entity,
            &TagData,
            &mut Transform,
            &GlobalTransform,
            &SelectedTag,
            Option<&TagMessagesData>,
        ),
        With<SelectedTag>,
    >,
    query_models: Query<(Entity, &ModelData)>,
    query_projects: Query<(Entity, &ProjectData, Option<&ThisProjectIsSelected>)>,
    mut tag_drafts: Local<HashMap<i32, TagDraft>>,
    // tag whose Delete was clicked, waiting for confirmation: (id, title)
    mut tag_to_delete: Local<Option<(i32, String)>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let current_selected_project = query_projects
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(entity, project_data, _)| (entity, project_data));

    for (entity, tag_data, mut transform, g_transform, _selected_tag, tag_messages) in
        &mut query_tags
    {
        // title and comment edits go into a draft, TagData stays what the server sent;
        // position is edited live on the Transform and compared with the saved one
        let saved = &tag_data.dto;
        let saved_position = Vec3::new(saved.position_x, saved.position_y, saved.position_z);
        let mut draft = tag_drafts
            .get(&saved.id)
            .cloned()
            .unwrap_or_else(|| TagDraft::from_dto(saved));

        // the window is shown while the tag has SelectedTag, X closes it like the Close button
        let mut window_open = true;
        egui::Window::new(tag_data.dto.title.clone())
            .id(Id::new(tag_data.dto.id))
            .open(&mut window_open)
            .show(ctx, |ui| {
                egui::Grid::new(Id::new("Tag grid 1"))
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(format!("Title: "));
                        ui.text_edit_singleline(&mut draft.title);
                        ui.end_row();

                        ui.label("Created by: ");
                        ui.label(&tag_data.dto.email);
                        ui.end_row();

                        match &tag_data.dto.status_dto {
                            Some(status_dto) => {
                                ui.label("Status: ");
                                status_widget(ui, status_dto);
                            }
                            None => {
                                ui.label("Status: ");
                                ui.label("No status");
                            }
                        }
                        ui.end_row();

                        ui.label("");
                        if ui.button("Set status").clicked() {
                            match current_selected_project {
                                Some(some) => {
                                    commands.trigger(GetStatusesEvent {
                                        project_id: some.1.dto.id,
                                    });
                                    commands.entity(entity).insert(TagHasOpenStatusModal {});
                                }
                                None => (),
                            };
                        }
                        ui.end_row();

                        ui.label("XYZ:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut transform.translation.x)
                                    .speed(0.01)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.add(
                                egui::DragValue::new(&mut transform.translation.y)
                                    .speed(0.01)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.add(
                                egui::DragValue::new(&mut transform.translation.z)
                                    .speed(0.01)
                                    .range(f32::MIN..=f32::MAX),
                            );
                        });

                        ui.end_row();

                        ui.label("New comment:");
                        ui.text_edit_multiline(&mut draft.comment);
                        ui.end_row();

                        let tag_changed = draft.title != saved.title
                            || transform.translation.distance(saved_position) > POSITION_EPSILON;
                        let has_comment = !draft.comment.trim().is_empty();

                        ui.horizontal(|ui| {
                            // only offer saving when something was edited
                            if tag_changed || has_comment {
                                let valid = !draft.title.trim().is_empty();
                                if ui
                                    .add_enabled(valid, egui::Button::new("Submit"))
                                    .on_disabled_hover_text("Title must not be empty")
                                    .clicked()
                                    && current_selected_project.is_some()
                                {
                                    if tag_changed {
                                        let mut tag_dto = saved.clone();
                                        tag_dto.title = draft.title.trim().to_string();
                                        tag_dto.position_x = g_transform.translation().x;
                                        tag_dto.position_y = g_transform.translation().y;
                                        tag_dto.position_z = g_transform.translation().z;
                                        commands.trigger(UpdateTagEvent { tag_dto });
                                    }
                                    if has_comment {
                                        commands.trigger(CreateTagMessageEvent {
                                            dto: NewTagMessageDto {
                                                text: draft.comment.clone(),
                                                tag_id: saved.id,
                                            },
                                        });
                                        draft.comment.clear();
                                    }
                                }
                            }
                            if tag_changed && ui.button("Revert").clicked() {
                                draft.title = saved.title.clone();
                                transform.translation = saved_position;
                            }
                        });
                        ui.end_row();

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            if ui.button("Delete").clicked() {
                                *tag_to_delete =
                                    Some((tag_data.dto.id, tag_data.dto.title.clone()));
                            }
                        });
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.heading("Comments:");
                    if ui.button("⟲").clicked() {
                        commands.trigger(GetTagMessagesEvent {
                            tag_id: tag_data.dto.id,
                        });
                    };
                });
                ui.vertical(|ui| {
                    ui.set_max_height(400.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(tag_messages) = tag_messages {
                            for tag_message in &tag_messages.dtos {
                                let mut cache = CommonMarkCache::default();
                                ui.vertical(|ui| {
                                    // ui.text_edit_multiline(&mut tag_message.text.as_ref());
                                    CommonMarkViewer::new().show(
                                        ui,
                                        &mut cache,
                                        tag_message.text.as_ref(),
                                    );
                                    ui.label(tag_message.email.clone());
                                    ui.separator();
                                    ui.separator();
                                });
                            }
                        }
                    });
                })
            });
        if !window_open {
            commands.entity(entity).remove::<SelectedTag>();
        }

        // a draft equal to the server data (saved or reverted) isn't needed anymore
        if draft.title == saved.title && draft.comment.is_empty() {
            tag_drafts.remove(&saved.id);
        } else {
            tag_drafts.insert(saved.id, draft);
        }
    }

    // one confirmation for all tag windows, for the tag whose Delete was clicked
    if let Some((tag_id, title)) = tag_to_delete.clone() {
        match confirm_modal(ctx, &format!("Delete tag \"{title}\"?"), &true, tag_id) {
            ConfirmModalResult::Confirm => {
                *tag_to_delete = None;
                commands.trigger(DeleteTagEvent { tag_id });
            }
            ConfirmModalResult::Cancel => {
                *tag_to_delete = None;
            }
            ConfirmModalResult::Nothing => (),
        }
    }
}

/// Tag position differences below this are treated as unchanged.
const POSITION_EPSILON: f32 = 1e-4;

/// Unsaved edits in a tag window.
#[derive(Clone)]
pub struct TagDraft {
    title: String,
    comment: String,
}

impl TagDraft {
    fn from_dto(dto: &TagDto) -> Self {
        Self {
            title: dto.title.clone(),
            comment: String::new(),
        }
    }
}

pub fn status_widget(ui: &mut egui::Ui, status_dto: &StatusDto) {
    let color = (
        convert_color_to_egui(status_dto.color_r),
        convert_color_to_egui(status_dto.color_g),
        convert_color_to_egui(status_dto.color_b),
    );

    let egui_color_bg = egui::Color32::from_rgba_unmultiplied(color.0, color.1, color.2, 30);
    let egui_color = egui::Color32::from_rgba_unmultiplied(color.0, color.1, color.2, 255);

    egui::Frame::default()
        .inner_margin(5.)
        .outer_margin(0.)
        .fill(egui_color_bg.clone())
        .stroke(egui::Stroke::new(1.0, egui_color.clone()))
        .corner_radius(CornerRadius::same(14))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(&status_dto.title).color(egui_color.clone()))
                    .truncate(),
            );
        });
}
