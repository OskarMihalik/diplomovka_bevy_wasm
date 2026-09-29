use bevy::prelude::*;
use bevy_egui::{
    egui::{self, vec2, Color32, Frame, Id, Stroke},
    EguiContexts,
};

use crate::{
    api::UpdateTagEvent,
    building::{KanbanOpen, ProjectStatusesData, TagData},
    utils::{compare_by_created_at, convert_color_to_egui},
};

use super::viewing_model::status_widget;

/// What is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    col: usize,
    row: usize,
}

pub fn kanban_window(
    mut commands: Commands,
    q_kanban: Query<(Entity, &KanbanOpen)>,
    mut contexts: EguiContexts,
    q_status: Query<&ProjectStatusesData>,
    q_tags: Query<&TagData>,
) {
    let Some(kanban) = q_kanban.iter().next() else {
        return;
    };
    let Some(statuses) = q_status.iter().next() else {
        return;
    };

    let statuses = &statuses.dtos;

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = true;
    egui::Window::new("Kanban").open(&mut open).show(ctx, |ui| {
        ui.heading("Kanban");
        ui.add_space(10.);
        // If there is a drop, store the location of the item being dragged, and the destination for the drop.
        let mut from = None;
        let mut to = None;

        ui.columns(statuses.len(), |uis| {
            for (col_idx, column) in statuses.iter().enumerate() {
                let ui = &mut uis[col_idx];
                let frame = Frame::default().inner_margin(4.0);

                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        status_widget(ui, column);

                        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                            ui.set_min_size(vec2(64.0, 100.0));
                            for (row_idx, item) in q_tags
                                .iter()
                                .filter(|tag| {
                                    tag.dto.status_dto.as_ref().map_or(-1, |status| status.id)
                                        == column.id
                                })
                                .enumerate()
                            {
                                let item_id = Id::new(("status_kanban", col_idx, row_idx));
                                let item_location = Location {
                                    col: col_idx,
                                    row: row_idx,
                                };
                                let response = ui
                                    .dnd_drag_source(item_id, item_location, |ui| {
                                        let _ = ui.button(format!(
                                            "{}: {}",
                                            item.dto.id,
                                            item.dto.title.clone()
                                        ));
                                    })
                                    .response;

                                // Detect drops onto this item:
                                if let (Some(pointer), Some(hovered_payload)) = (
                                    ui.input(|i| i.pointer.interact_pos()),
                                    response.dnd_hover_payload::<Location>(),
                                ) {
                                    let rect = response.rect;

                                    // Preview insertion:
                                    let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                                    let insert_row_idx = if *hovered_payload == item_location {
                                        // We are dragged onto ourselves
                                        ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                        row_idx
                                    } else if pointer.y < rect.center().y {
                                        // Above us
                                        ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                        row_idx
                                    } else {
                                        // Below us
                                        ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                        row_idx + 1
                                    };

                                    if let Some(dragged_payload) = response.dnd_release_payload() {
                                        // The user dropped onto this item.
                                        from = Some(dragged_payload);
                                        to = Some(Location {
                                            col: col_idx,
                                            row: insert_row_idx,
                                        });
                                        bevy::log::info!(
                                            "Dropped 1 {:#?} on {:#?}",
                                            from.clone(),
                                            to
                                        );
                                    }
                                }
                            }
                        });

                        if let Some(dragged_payload) = dropped_payload {
                            // The user dropped onto the column, but not on any one item.
                            from = Some(dragged_payload);
                            to = Some(Location {
                                col: col_idx,
                                row: usize::MAX, // Inset last
                            });
                            bevy::log::info!("Dropped 2 {:#?} on {:#?}", from.clone(), to);
                        }
                    },
                );
            }
        });

        if let (Some(from), Some(mut to)) = (from, to) {
            if from.col == to.col {
                // Dragging within the same column.
                // Adjust row index if we are re-ordering:
                to.row -= (from.row < to.row) as usize;
            }
            bevy::log::info!("Dropped 3 {:#?} on {:#?}", from.clone(), to);
            let status = statuses[to.col].clone();
            let old_status_id = statuses[from.col].id;
            let mut new_tag = match q_tags
                .iter()
                // .find(|tag| tag.dto.status_dto.as_ref().map_or(-1, |sta| sta.id) == old_status_id)
                .filter(|tag| {
                    tag.dto.status_dto.as_ref().map_or(-1, |status| status.id) == old_status_id
                })
                .nth(from.row)
            {
                Some(data) => data.dto.clone(),
                None => return,
            };
            new_tag.status_dto = Some(status.clone());
            commands.trigger(UpdateTagEvent { tag_dto: new_tag });
        }
    });

    if open == false {
        commands.entity(kanban.0).despawn();
    }
}
