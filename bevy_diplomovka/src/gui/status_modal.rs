use crate::{
    api::{CreateStatusEvent, DeleteStatusEvent, UpdateStatusEvent, UpdateTagEvent},
    building::{
        ModelData, ProjectData, ProjectStatusesData, TagData, TagHasOpenStatusModal,
        ThisModelIsSelected, ThisProjectIsSelected,
    },
};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Id},
    EguiContexts,
};
use dto::default::{NewStatusDto, StatusDto};
use egui_extras::{Column, TableBuilder};

use super::viewing_model::status_widget;

pub fn ui_status_modal(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut query_statuses: Query<&mut ProjectStatusesData>,
    query_tags: Query<(Entity, &TagData, &TagHasOpenStatusModal)>,
    selected_model_query: Option<Single<(Entity, &ModelData, &ThisModelIsSelected)>>,
    mut new_status: Local<NewStatusDto>,
    selected_project_query: Single<(Entity, &ProjectData, &ThisProjectIsSelected)>,
) {
    let ctx = contexts.ctx_mut();

    let tag = match query_tags.iter().next() {
        Some(tag) => tag,
        None => return,
    };

    let Some(selected_model) = selected_model_query else {
        return;
    };

    for mut statuses in query_statuses.iter_mut() {
        let modal = egui::Modal::new(Id::new("Status")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Preview");
                    status_widget(
                        ui,
                        &StatusDto {
                            title: new_status.title.clone(),
                            id: 0,
                            color_r: new_status.color_r,
                            color_g: new_status.color_g,
                            color_b: new_status.color_b,
                            project_id: 0,
                        },
                    );
                });
                ui.vertical(|ui| {
                    ui.label("Title");
                    ui.text_edit_singleline(&mut new_status.title)
                });
                ui.vertical(|ui| {
                    ui.label("R");
                    ui.add(
                        egui::DragValue::new(&mut new_status.color_r)
                            .speed(0.01)
                            .range(1.0..=0.0),
                    );
                });
                ui.vertical(|ui| {
                    ui.label("G");
                    ui.add(
                        egui::DragValue::new(&mut new_status.color_g)
                            .speed(0.01)
                            .range(1.0..=0.0),
                    );
                });
                ui.vertical(|ui| {
                    ui.label("B");
                    ui.add(
                        egui::DragValue::new(&mut new_status.color_b)
                            .speed(0.01)
                            .range(1.0..=0.0),
                    );
                });
                if ui.button("Add new status").clicked() {
                    new_status.project_id = selected_project_query.1.dto.id;
                    commands.trigger(CreateStatusEvent {
                        dto: new_status.clone(),
                    });
                }
            });

            let available_height = ui.available_height();

            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("Title");
                    });
                    header.col(|ui| {
                        ui.strong("R");
                    });
                    header.col(|ui| {
                        ui.strong("G");
                    });
                    header.col(|ui| {
                        ui.strong("B");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                })
                .body(|mut body| {
                    let row_height = 18.0;
                    for status in statuses.dtos.iter_mut() {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                status_widget(ui, status);
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut status.title);
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut status.color_r)
                                        .speed(0.01)
                                        .range(1.0..=0.0),
                                );
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut status.color_g)
                                        .speed(0.01)
                                        .range(1.0..=0.0),
                                );
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut status.color_b)
                                        .speed(0.01)
                                        .range(1.0..=0.0),
                                );
                            });
                            row.col(|ui| {
                                if ui.button("Edit").clicked() {
                                    // select this status
                                    commands.trigger(UpdateStatusEvent {
                                        dto: status.clone(),
                                    });
                                }
                            });
                            row.col(|ui| {
                                let checked = tag
                                    .1
                                    .dto
                                    .status_dto
                                    .as_ref()
                                    .map_or(false, |s| s.id == status.id);
                                if ui.selectable_label(checked, "Select").clicked() {
                                    let mut new_tag = tag.1.dto.clone();
                                    new_tag.status_dto = Some(status.clone());

                                    // select this status
                                    commands.trigger(UpdateTagEvent {
                                        tag_dto: new_tag,
                                        parent_entity: selected_model.0,
                                    });
                                }
                            });
                            row.col(|ui| {
                                if ui.button("🗙").clicked() {
                                    commands.trigger(DeleteStatusEvent {
                                        status_id: status.id,
                                        project_id: selected_project_query.1.dto.id,
                                    });
                                }
                            });
                        });
                    }
                });
            ui.add_space(100.);
        });
        if modal.should_close() {
            commands.entity(tag.0).remove::<TagHasOpenStatusModal>();
        }
    }
}
