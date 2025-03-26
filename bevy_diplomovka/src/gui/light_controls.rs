use bevy::{color::palettes::css::YELLOW, prelude::*};
use bevy_egui::{
    egui::{self, color_picker::Alpha},
    EguiContexts,
};

use crate::building::LightControlsOpen;

// commands.spawn((
//     PointLight {
//         shadows_enabled: true,
//         ..default()
//     },
//     Transform::from_xyz(4.0, 8.0, 4.0),
// ));

pub fn on_light_gizmos_added(
    mut config_store: ResMut<GizmoConfigStore>,
    q_light_controls: Query<(Entity, Option<&LightControlsOpen>), (Changed<LightControlsOpen>)>,
) {
    for light_control in q_light_controls.iter() {
        bevy::log::info!("is some: {}", light_control.1.is_some());
        let (_, light_config) = config_store.config_mut::<LightGizmoConfigGroup>();
        light_config.color = LightGizmoColor::MatchLightColor;
        light_config.draw_all = true;
    }
}

pub fn on_light_gizmos_removed(
    mut config_store: ResMut<GizmoConfigStore>,
    mut removals: RemovedComponents<LightControlsOpen>,
) {
    for _ in removals.read() {
        // do something with the entity
        let (_, light_config) = config_store.config_mut::<LightGizmoConfigGroup>();
        light_config.color = LightGizmoColor::MatchLightColor;
        light_config.draw_all = false;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LightType {
    #[default]
    Point,
    Spot,
}

pub fn light_controls_window(
    mut commands: Commands,
    q_light_controls: Query<(Entity, &LightControlsOpen)>,
    mut contexts: EguiContexts,
    mut q_point_lights: Query<(Entity, &mut PointLight, &mut Transform), (Without<SpotLight>)>,
    mut q_spot_lights: Query<(Entity, &mut SpotLight, &mut Transform), (Without<PointLight>)>,
    mut new_light_type: Local<LightType>,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let Some(light_controls_open) = q_light_controls.iter().next() else {
        return;
    };

    let ctx = contexts.ctx_mut();
    let mut open = true;
    egui::Window::new("Light controls")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.add_space(10.);
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", *new_light_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut *new_light_type, LightType::Point, "Point Light");
                        ui.selectable_value(&mut *new_light_type, LightType::Spot, "Spot Light");
                    });
                if ui.button("Add Light").clicked() {
                    match *new_light_type {
                        LightType::Point => {
                            commands.spawn((
                                PointLight {
                                    ..Default::default()
                                },
                                Transform::from_xyz(4.0, 8.0, 4.0),
                            ));
                        }
                        LightType::Spot => {
                            commands.spawn((
                                SpotLight {
                                    ..Default::default()
                                },
                                Transform::from_xyz(4.0, 8.0, 4.0),
                            ));
                        }
                    }
                }
            });
            ui.add_space(10.);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for (entity, mut point_light, mut transform) in q_point_lights.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let light_c = point_light.color.to_srgba();
                            let mut color = egui::Color32::from_rgba_unmultiplied(
                                (light_c.red * 255.0) as u8,
                                (light_c.green * 255.0) as u8,
                                (light_c.blue * 255.0) as u8,
                                (light_c.alpha * 255.0) as u8,
                            );
                            egui::widgets::color_picker::color_edit_button_srgba(
                                ui,
                                &mut color,
                                Alpha::BlendOrAdditive,
                            );
                            point_light.color = Color::srgba(
                                color.r() as f32 / 255.0,
                                color.g() as f32 / 255.0,
                                color.b() as f32 / 255.0,
                                color.a() as f32 / 255.0,
                            );

                            ui.label("XYZ");
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

                            ui.label("Intensity: ");
                            ui.add(
                                egui::DragValue::new(&mut point_light.intensity)
                                    .speed(100)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Radius: ");
                            ui.add(
                                egui::DragValue::new(&mut point_light.radius)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Range: ");
                            ui.add(
                                egui::DragValue::new(&mut point_light.range)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            if ui.button("🗙").clicked() {
                                commands.entity(entity).despawn_recursive();
                            }
                        });
                        ui.separator();
                    }
                    for (entity, mut spot_light, mut transform) in q_spot_lights.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let light_c = spot_light.color.to_srgba();
                            let mut color = egui::Color32::from_rgba_unmultiplied(
                                (light_c.red * 255.0) as u8,
                                (light_c.green * 255.0) as u8,
                                (light_c.blue * 255.0) as u8,
                                (light_c.alpha * 255.0) as u8,
                            );
                            egui::widgets::color_picker::color_edit_button_srgba(
                                ui,
                                &mut color,
                                Alpha::BlendOrAdditive,
                            );
                            spot_light.color = Color::srgba(
                                color.r() as f32 / 255.0,
                                color.g() as f32 / 255.0,
                                color.b() as f32 / 255.0,
                                color.a() as f32 / 255.0,
                            );

                            ui.label("XYZ");
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

                            ui.label("Rotation: ");
                            let mut rot = transform.rotation.to_euler(EulerRot::YXZ);
                            ui.add(
                                egui::DragValue::new(&mut rot.0)
                                    .speed(0.01)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.add(
                                egui::DragValue::new(&mut rot.1)
                                    .speed(0.01)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.add(
                                egui::DragValue::new(&mut rot.2)
                                    .speed(0.01)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            transform.rotation =
                                Quat::from_euler(EulerRot::YXZ, rot.0, rot.1, rot.2);

                            ui.label("Intensity: ");
                            ui.add(
                                egui::DragValue::new(&mut spot_light.intensity)
                                    .speed(100)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Radius: ");
                            ui.add(
                                egui::DragValue::new(&mut spot_light.radius)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.label("Range: ");
                            ui.add(
                                egui::DragValue::new(&mut spot_light.range)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            if ui.button("🗙").clicked() {
                                commands.entity(entity).despawn_recursive();
                            }
                        });
                        ui.separator();
                    }
                });
            });
        });

    if open == false {
        commands.entity(light_controls_open.0).despawn_recursive();
    }
}
