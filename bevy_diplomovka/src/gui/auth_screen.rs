use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use dto::auth::{LoginDto, RegisterDto};
use egui_form::{garde::GardeReport, Form, FormField};
use garde::Validate;

use crate::api::{LoginEvent, RegisterEvent};

use super::form::{enter_pressed, submit_button, text_input};

const FORM_WIDTH: f32 = 300.0;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum AuthType {
    #[default]
    Login,
    Register,
}

#[derive(Validate, Default, Clone)]
pub struct LoginForm {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 1))]
    pub password: String,
}

#[derive(Validate, Default, Clone)]
pub struct RegisterForm {
    // keep in sync with backend-express/src/controllers/auth.ts
    #[garde(length(min = 3))]
    pub username: String,
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 8))]
    pub password: String,
    #[garde(matches(password))]
    pub password_repeat: String,
}

/// Auth form state. Inputs are kept between frames and after failed submits,
/// and cleared only when leaving the auth screen (see [`reset_auth_forms`]).
#[derive(Resource, Default)]
pub struct AuthForms {
    pub auth_type: AuthType,
    pub login: LoginForm,
    pub register: RegisterForm,
    /// Bumped on reset so egui_form forgets which fields were already touched.
    generation: u32,
}

pub fn reset_auth_forms(mut forms: ResMut<AuthForms>) {
    let generation = forms.generation.wrapping_add(1);
    *forms = AuthForms {
        generation,
        ..default()
    };
}

pub fn login_screen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut forms: ResMut<AuthForms>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let forms = &mut *forms;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.allocate_ui(egui::vec2(FORM_WIDTH, 0.0), |ui| {
                ui.push_id(
                    (forms.generation, forms.auth_type as u8),
                    |ui| match forms.auth_type {
                        AuthType::Login => login_form(ui, &mut commands, forms),
                        AuthType::Register => register_form(ui, &mut commands, forms),
                    },
                );
            });
        });
    });
}

fn login_form(ui: &mut egui::Ui, commands: &mut Commands, forms: &mut AuthForms) {
    let login = &mut forms.login;
    let mut form = Form::new().add_report(GardeReport::new(login.validate()));

    ui.heading("Login");
    ui.add_space(20.0);

    let email = FormField::new(&mut form, "email")
        .label("Email")
        .ui(ui, text_input(&mut login.email));
    let password = FormField::new(&mut form, "password")
        .label("Password")
        .ui(ui, text_input(&mut login.password).password(true));

    ui.add_space(10.0);
    let submit = submit_button(ui, "Log in");

    if (submit.clicked() || enter_pressed(ui, &[&email, &password])) && form.try_submit(ui).is_ok()
    {
        commands.trigger(LoginEvent {
            dto: LoginDto {
                email: login.email.clone(),
                password: login.password.clone(),
            },
        });
    }

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.label("Don't have an account?");
        if ui.link("Register").clicked() {
            // carry the email over so it doesn't have to be typed twice
            if forms.register.email.is_empty() {
                forms.register.email = forms.login.email.clone();
            }
            forms.auth_type = AuthType::Register;
        }
    });
}

fn register_form(ui: &mut egui::Ui, commands: &mut Commands, forms: &mut AuthForms) {
    let register = &mut forms.register;
    let mut form = Form::new().add_report(GardeReport::new(register.validate()));

    ui.heading("Register");
    ui.add_space(20.0);

    let username = FormField::new(&mut form, "username")
        .label("Username")
        .ui(ui, text_input(&mut register.username));
    let email = FormField::new(&mut form, "email")
        .label("Email")
        .ui(ui, text_input(&mut register.email));
    let password = FormField::new(&mut form, "password")
        .label("Password")
        .ui(ui, text_input(&mut register.password).password(true));
    let password_repeat = FormField::new(&mut form, "password_repeat")
        .label("Repeat password")
        .ui(ui, text_input(&mut register.password_repeat).password(true));

    ui.add_space(10.0);
    let submit = submit_button(ui, "Create account");

    let fields = [&username, &email, &password, &password_repeat];
    if (submit.clicked() || enter_pressed(ui, &fields)) && form.try_submit(ui).is_ok() {
        commands.trigger(RegisterEvent {
            dto: RegisterDto {
                username: register.username.clone(),
                email: register.email.clone(),
                password: register.password.clone(),
            },
        });
    }

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.label("Already have an account?");
        if ui.link("Log in").clicked() {
            if forms.login.email.is_empty() {
                forms.login.email = forms.register.email.clone();
            }
            forms.auth_type = AuthType::Login;
        }
    });
}
