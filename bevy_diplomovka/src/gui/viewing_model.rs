use bevy::prelude::*;
use bevy_egui::{
    egui::{
        self,
        text::{LayoutJob, TextWrapping},
        Id, Rounding, ScrollArea, TextFormat, TextStyle,
    },
    EguiContexts,
};
use dto::default::{NewTagMessageDto, StatusDto};

use crate::{
    api::{
        CreateTagMessageEvent, DeleteTagEvent, GetStatusesEvent, GetTagMessagesEvent,
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
    trigger: Trigger<FilterChangeEvent>,
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
) {
    let ctx = contexts.ctx_mut();
    let kanban = q_kanban_open.iter().next();
    let light_controls = q_light_controls.iter().next();
    egui::SidePanel::left("left_panel")
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
                                commands.entity(entity).despawn_recursive();
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
                                commands.entity(entity).despawn_recursive();
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
            &mut TagData,
            &mut Transform,
            &GlobalTransform,
            &SelectedTag,
            Option<&TagMessagesData>,
        ),
        With<SelectedTag>,
    >,
    query_models: Query<(Entity, &ModelData)>,
    query_projects: Query<(Entity, &ProjectData, Option<&ThisProjectIsSelected>)>,
    mut new_message_text: Local<String>,
    mut confirm_delete_tag_modal_open: Local<bool>,
) {
    let ctx = contexts.ctx_mut();

    let current_selected_project = query_projects
        .iter()
        .find(|(_, _, selected)| selected.is_some())
        .map(|(entity, project_data, _)| (entity, project_data));

    for (entity, mut tag_data, mut transform, g_transform, _selected_tag, tag_messages) in
        &mut query_tags
    {
        egui::Window::new(tag_data.dto.title.clone())
            .id(Id::new(tag_data.dto.id))
            .show(ctx, |ui| {
                egui::Grid::new(Id::new("Tag grid 1"))
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(format!("Title: "));
                        ui.text_edit_singleline(&mut tag_data.dto.title);
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
                        ui.text_edit_multiline(&mut *new_message_text);
                        ui.end_row();

                        if ui.button("Submit").clicked() {
                            if let Some((_, _model_data)) = current_selected_project {
                                let mut tag_dto = tag_data.dto.clone();
                                tag_dto.position_x = g_transform.translation().x;
                                tag_dto.position_y = g_transform.translation().y;
                                tag_dto.position_z = g_transform.translation().z;
                                commands.trigger(UpdateTagEvent { tag_dto });
                                if !new_message_text.is_empty() {
                                    commands.trigger(CreateTagMessageEvent {
                                        dto: NewTagMessageDto {
                                            text: new_message_text.clone(),
                                            tag_id: tag_data.dto.id,
                                        },
                                    });
                                    *new_message_text = String::new();
                                }
                            }
                        }
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            if ui.button("Close").clicked() {
                                commands.entity(entity).remove::<SelectedTag>();
                            }
                            if ui.button("Delete").clicked() {
                                *confirm_delete_tag_modal_open = true;
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

        if *confirm_delete_tag_modal_open {
            match confirm_modal(
                ctx,
                "Delete?",
                &confirm_delete_tag_modal_open,
                tag_data.dto.id,
            ) {
                ConfirmModalResult::Confirm => {
                    *confirm_delete_tag_modal_open = false;
                    commands.trigger(DeleteTagEvent {
                        tag_id: tag_data.dto.id,
                    });
                }
                ConfirmModalResult::Cancel => {
                    *confirm_delete_tag_modal_open = false;
                }
                ConfirmModalResult::Nothing => (),
            }
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
    // Put the buttons and label on the same row:
    egui::Frame::default()
        .inner_margin(5.)
        .outer_margin(0.)
        .fill(egui_color_bg.clone())
        .stroke(egui::Stroke::new(1.0, egui_color.clone()))
        .rounding(Rounding {
            nw: 14.,
            ne: 14.,
            sw: 14.,
            se: 14.,
        })
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(&status_dto.title).color(egui_color.clone()))
                    .truncate(),
            );
        });
}
