use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Id, ScrollArea},
    EguiContexts,
};
use bevy_file_dialog::prelude::*;

use crate::{
    api::UpdateTagEvent,
    building::{ModelData, SelectedTag, TagData},
    utils::compare_by_created_at,
    GameState,
};

use super::gui::GlbFileContents;

pub fn ui_viewing_model(
    mut commands: Commands,
    mut contexts: EguiContexts,
    query_tags: Query<(Entity, &TagData, Option<&SelectedTag>)>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Some things");
            if ui.button("Select model").clicked() {
                commands.set_state(GameState::SelectingProjectAndModel);
            }
            if ui.add(egui::widgets::Button::new("Load model")).clicked() {
                commands
                    .dialog()
                    .add_filter("Glb", &["glb"])
                    .load_file::<GlbFileContents>();
            }

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
        ),
        With<SelectedTag>,
    >,
    query_models: Query<(Entity, &ModelData)>,
) {
    let ctx = contexts.ctx_mut();
    for (entity, mut tag_data, mut transform, g_transform, _selected_tag) in &mut query_tags {
        egui::Window::new(tag_data.dto.title.clone())
            .id(Id::new(tag_data.dto.id))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Title: "));
                    ui.text_edit_singleline(&mut tag_data.dto.title);
                });
                ui.horizontal(|ui| {
                    ui.label("XYZ:");
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

                if ui.button("Submit").clicked() {
                    let parent_entity = query_models
                        .iter()
                        .find(|model| model.1.dto.id == tag_data.dto.model_id);
                    if let Some((target_entity, _model_data)) = parent_entity {
                        let mut tag_dto = tag_data.dto.clone();
                        tag_dto.position_x = g_transform.translation().x;
                        tag_dto.position_y = g_transform.translation().y;
                        tag_dto.position_z = g_transform.translation().z;
                        commands.trigger(UpdateTagEvent {
                            tag_dto,
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
