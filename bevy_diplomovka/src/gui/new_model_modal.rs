use bevy::prelude::*;
use bevy_egui::egui;
use egui_form::{garde::GardeReport, Form, FormField};
use garde::Validate;

use crate::{api::CreateModelEvent, api_tracking::ApiStatus};

use super::{
    form::{enter_pressed, not_blank, secondary_button, submit_button, text_input},
    glb_picker::open_glb_picker,
    gui::UploadedGlbFile,
};

const FORM_WIDTH: f32 = 300.0;

#[derive(Validate, Default, Clone)]
pub struct NewModelForm {
    #[garde(custom(not_blank))]
    pub name: String,
    /// Name of the loaded glb file, mirrored from [`UploadedGlbFile`] every frame.
    #[garde(custom(file_chosen))]
    pub file_name: Option<String>,
}

fn file_chosen(value: &Option<String>, _ctx: &()) -> garde::Result {
    match value {
        Some(_) => Ok(()),
        None => Err(garde::Error::new("choose a .glb file")),
    }
}

#[derive(Default)]
pub struct NewModelModal {
    pub open: bool,
    pub form: NewModelForm,
    /// Bumped on reset so egui_form forgets which fields were already touched.
    generation: u32,
}

impl NewModelModal {
    fn close_and_reset(&mut self) {
        *self = Self {
            generation: self.generation.wrapping_add(1),
            ..default()
        };
    }
}

/// "Add new model" modal. Validates the form before uploading and closes
/// only after the server confirmed the model was created; on failure it
/// stays open with the name and the loaded file so it can be retried.
pub fn new_model_modal(
    ctx: &egui::Context,
    commands: &mut Commands,
    modal: &mut NewModelModal,
    status: &mut ApiStatus<CreateModelEvent>,
    project_id: Option<i32>,
    glb_file: Option<(Entity, &UploadedGlbFile)>,
) {
    if status.succeeded() {
        modal.close_and_reset();
        if let Some((entity, _)) = glb_file {
            commands.entity(entity).despawn();
        }
    }
    if !modal.open {
        return;
    }
    let pending = status.is_pending();

    let NewModelModal {
        open,
        form: model,
        generation,
    } = modal;
    model.file_name = glb_file.map(|(_, file)| file.file_name.clone());

    let response = egui::Modal::new(egui::Id::new("Add new model modal")).show(ctx, |ui| {
        ui.set_width(FORM_WIDTH);
        ui.heading("Add new model");
        ui.add_space(10.0);
        ui.push_id(*generation, |ui| {
            let mut form = Form::new().add_report(GardeReport::new(model.validate()));

            // inputs are locked while waiting for the server
            let name = ui
                .add_enabled_ui(!pending, |ui| {
                    let name = FormField::new(&mut form, "name")
                        .label("Model name")
                        .ui(ui, text_input(&mut model.name));

                    let file_button_text = match &model.file_name {
                        Some(file_name) => format!("📄 {file_name}"),
                        None => "Choose .glb file…".to_string(),
                    };
                    let file_button = FormField::new(&mut form, "file_name")
                        .label("GLB file")
                        .ui(ui, egui::Button::new(file_button_text));
                    if file_button.clicked() {
                        open_glb_picker(commands);
                    }
                    name
                })
                .inner;

            ui.add_space(10.0);
            let submit = ui
                .add_enabled_ui(!pending, |ui| submit_button(ui, "Upload model"))
                .inner;
            let cancel = ui
                .add_enabled_ui(!pending, |ui| secondary_button(ui, "Cancel"))
                .inner;
            if pending {
                ui.vertical_centered(|ui| ui.spinner());
            }

            if !pending
                && (submit.clicked() || enter_pressed(ui, &[&name]))
                && form.try_submit(ui).is_ok()
            {
                match (project_id, glb_file) {
                    (Some(project_id), Some((_, file))) => {
                        commands.trigger(CreateModelEvent {
                            name: model.name.trim().to_string(),
                            project_id,
                            model_bytes: file.contents.clone(),
                        });
                    }
                    _ => bevy::log::error!("No project selected"),
                }
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
