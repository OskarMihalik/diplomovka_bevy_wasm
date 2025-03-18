use bevy::prelude::*;
use bevy_egui::{
    egui::{self, vec2, Color32, Frame, Id},
    EguiContexts,
};

use crate::building::{KanbanOpen, ProjectStatusesData, TagData};

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

    let ctx = contexts.ctx_mut();
    let mut open = true;
    egui::Window::new("Kanban").open(&mut open).show(ctx, |ui| {
        ui.heading("Kanban");
        ui.label("This is a simple example of drag-and-drop in egui.");
        ui.label("Drag items between columns.");

        // If there is a drop, store the location of the item being dragged, and the destination for the drop.
        let mut from = None;
        let mut to = None;

        ui.columns(statuses.len(), |uis| {
            for (col_idx, column) in statuses.iter().enumerate() {
                let ui = &mut uis[col_idx];

                let frame = Frame::default().inner_margin(4.0);

                let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                    ui.set_min_size(vec2(64.0, 100.0));
                    for (row_idx, item) in q_tags
                        .iter()
                        .filter(|tag| {
                            tag.dto.status_dto.as_ref().map_or(-1, |status| status.id) == column.id
                        })
                        .enumerate()
                    {
                        let item_id = Id::new(("my_drag_and_drop_demo", col_idx, row_idx));
                        let item_location = Location {
                            col: col_idx,
                            row: row_idx,
                        };
                        let response = ui
                            .dnd_drag_source(item_id, item_location, |ui| {
                                ui.label(item.dto.title.clone());
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
                }
            }
        });

        if let (Some(from), Some(mut to)) = (from, to) {
            if from.col == to.col {
                // Dragging within the same column.
                // Adjust row index if we are re-ordering:
                to.row -= (from.row < to.row) as usize;
            }

            // let item = statuses[from.col].remove(from.row);

            // let column = &mut self.columns[to.col];
            // to.row = to.row.min(column.len());
            // column.insert(to.row, item);
        }
    });

    if open == false {
        commands.entity(kanban.0).despawn_recursive();
    }
}
