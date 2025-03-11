use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Id},
    EguiContexts,
};

use crate::building::{ProjectStatusesData, TagData, TagHasOpenStatusModal};
use egui_extras::{Column, TableBuilder};

use super::viewing_model::status_widget;

pub fn ui_status_modal(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut query_statuses: Query<&mut ProjectStatusesData>,
    mut query_tags: Query<(Entity, &TagData, &TagHasOpenStatusModal)>,
) {
    let ctx = contexts.ctx_mut();

    let tag = match query_tags.iter().next() {
        Some(tag) => tag,
        None => return,
    };

    for mut statuses in query_statuses.iter_mut() {
        let modal = egui::Modal::new(Id::new("Status")).show(ctx, |ui| {
            let available_height = ui.available_height();
            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(
                    Column::remainder()
                        .at_least(40.0)
                        .clip(true)
                        .resizable(true),
                )
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::remainder())
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            table
                .header(20.0, |mut header| {
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
                })
                .body(|mut body| {
                    let row_height = 18.0;
                    for status in statuses.dtos.iter_mut() {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                status_widget(ui, status);
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
                                }
                            });
                        });
                    }
                })
        });
        if modal.should_close() {
            commands.entity(tag.0).remove::<TagHasOpenStatusModal>();
        }
    }
}
