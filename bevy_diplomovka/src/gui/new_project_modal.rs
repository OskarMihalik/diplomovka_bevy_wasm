use bevy::prelude::*;
use bevy_egui::egui;
use dto::project::NewProjectDto;
use egui_form::{garde::GardeReport, Form, FormField};
use garde::Validate;

use crate::{api::NewProjectEvent, api_tracking::ApiStatus};

use super::form::{enter_pressed, not_blank, submit_button, text_area, text_input};

const FORM_WIDTH: f32 = 300.0;

#[derive(Validate, Default, Clone)]
pub struct NewProjectForm {
    #[garde(custom(not_blank))]
    pub name: String,
    #[garde(skip)]
    pub description: String,
}

#[derive(Default)]
pub struct NewProjectModal {
    pub open: bool,
    pub form: NewProjectForm,
    /// Bumped on reset so egui_form forgets which fields were already touched.
    generation: u32,
}

impl NewProjectModal {
    fn close_and_reset(&mut self) {
        *self = Self {
            generation: self.generation.wrapping_add(1),
            ..default()
        };
    }
}

/// "Add new project" window. Validates the form before sending and closes
/// only after the server confirmed the project was created; on failure it
/// stays open with the inputs so they can be fixed.
pub fn new_project_modal(
    ctx: &egui::Context,
    commands: &mut Commands,
    modal: &mut NewProjectModal,
    status: &mut ApiStatus<NewProjectEvent>,
) {
    if status.succeeded() {
        modal.close_and_reset();
    }
    if !modal.open {
        return;
    }
    let pending = status.is_pending();

    let NewProjectModal {
        open,
        form: project,
        generation,
    } = modal;

    let response = egui::Modal::new(egui::Id::new("Add new project modal")).show(ctx, |ui| {
        ui.set_width(FORM_WIDTH);
        ui.heading("Add new project");
        ui.add_space(10.0);
        ui.push_id(*generation, |ui| {
            let mut form = Form::new().add_report(GardeReport::new(project.validate()));

            // inputs are locked while waiting for the server
            let name = ui
                .add_enabled_ui(!pending, |ui| {
                    let name = FormField::new(&mut form, "name")
                        .label("Name")
                        .ui(ui, text_input(&mut project.name));
                    FormField::new(&mut form, "description")
                        .label("Description")
                        .ui(ui, text_area(&mut project.description));
                    name
                })
                .inner;

            ui.add_space(10.0);
            let submit = ui
                .add_enabled_ui(!pending, |ui| submit_button(ui, "Create project"))
                .inner;
            let cancel = ui
                .add_enabled_ui(!pending, |ui| {
                    ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Cancel"))
                })
                .inner;
            if pending {
                ui.vertical_centered(|ui| ui.spinner());
            }

            if !pending
                && (submit.clicked() || enter_pressed(ui, &[&name]))
                && form.try_submit(ui).is_ok()
            {
                commands.trigger(NewProjectEvent {
                    dto: NewProjectDto {
                        name: project.name.trim().to_string(),
                        description: project.description.clone(),
                    },
                });
            }

            cancel.clicked()
        })
        .inner
    });

    // Esc / click outside also close it, but not while waiting for the server
    if !pending && (response.inner || response.should_close()) {
        *open = false;
    }
}
