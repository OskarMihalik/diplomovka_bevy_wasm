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

pub fn light_controls_window(
    mut commands: Commands,
    q_light_controls: Query<(Entity, &LightControlsOpen)>,
    mut contexts: EguiContexts,
    mut q_point_lights: Query<(Entity, &mut PointLight, &mut Transform)>,
    mut gizmos: Gizmos,
) {
    let Some(light_controls_open) = q_light_controls.iter().next() else {
        return;
    };

    let ctx = contexts.ctx_mut();
    let mut open = true;
    egui::Window::new("Light controls")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Light controls");
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

                            ui.label("Rotation: ");
                            let mut rot = transform.rotation.clone().to_euler(EulerRot::YXZ);
                            ui.add(
                                egui::DragValue::new(&mut rot.0)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.add(
                                egui::DragValue::new(&mut rot.1)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            ui.add(
                                egui::DragValue::new(&mut rot.2)
                                    .speed(0.1)
                                    .range(f32::MIN..=f32::MAX),
                            );
                            transform.rotation = Quat::from_euler(
                                EulerRot::YXZ,
                                rot.0.to_radians(),
                                rot.1.to_radians(),
                                rot.2.to_radians(),
                            );

                            ui.label("Intensity: ");

                            if ui.button("🗙").clicked() {
                                commands.entity(entity).despawn_recursive();
                            }
                            let gizmo_pos = transform.translation.clone();
                            let gizmo_rot = transform.rotation.clone();
                            gizmos.cuboid(
                                Transform::from_translation(gizmo_pos)
                                    .with_rotation(gizmo_rot)
                                    .with_scale(Vec3::new(0.5, 1., 0.5)),
                                point_light.color,
                            );
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
