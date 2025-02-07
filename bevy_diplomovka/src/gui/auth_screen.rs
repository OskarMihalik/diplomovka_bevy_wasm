use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use dto::auth::{LoginDto, RegisterDto};

use crate::api::{LoginEvent, RegisterEvent};

#[derive(Default)]
pub enum AuthType {
    #[default]
    Login,
    Register,
}

#[derive(Default)]

pub struct LocalAuthType {
    pub auth_type: AuthType,
}

pub fn login_screen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut login_dto: Local<LoginDto>,
    mut register_dto: Local<RegisterDto>,
    mut auth_type: Local<LocalAuthType>,
) {
    let ctx = contexts.ctx_mut();
    let _ = match auth_type.auth_type {
        AuthType::Login => egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                ui.heading("Login");

                ui.add_space(20.0);

                ui.label("Email:");
                ui.text_edit_singleline(&mut login_dto.email);

                ui.add_space(10.0);

                ui.label("Password:");
                // ui.text_edit_singleline(&mut login_dto.password);
                ui.add(egui::TextEdit::singleline(&mut login_dto.password).password(true));

                ui.add_space(20.0);

                if ui.button("Submit").clicked() {
                    // Handle login logic here
                    commands.trigger(LoginEvent {
                        dto: (*login_dto).clone(),
                    });
                    login_dto.reset();
                }
                ui.add_space(30.0);
                if ui.button("Register").clicked() {
                    auth_type.auth_type = AuthType::Register
                }
            });
        }),
        AuthType::Register => egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Register");
                ui.add_space(20.0);

                ui.label("Username:");
                ui.text_edit_singleline(&mut register_dto.username);

                ui.add_space(20.0);

                ui.label("Email:");
                ui.text_edit_singleline(&mut register_dto.email);

                ui.add_space(10.0);

                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut register_dto.password).password(true));

                ui.label("Repeat password:");
                ui.add(egui::TextEdit::singleline(&mut register_dto.password).password(true));

                ui.add_space(20.0);

                if ui.button("Submit").clicked() {
                    // Handle login logic here
                    commands.trigger(RegisterEvent {
                        dto: (*register_dto).clone(),
                    });
                    register_dto.reset();
                }
                ui.add_space(30.0);
                if ui.button("Register").clicked() {
                    auth_type.auth_type = AuthType::Register
                }
            });
        }),
    };
}
