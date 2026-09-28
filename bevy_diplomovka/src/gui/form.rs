//! Shared helpers for egui_form + garde forms.

use bevy_egui::egui;

pub fn text_input(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text).desired_width(f32::INFINITY)
}

pub fn text_area(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::multiline(text).desired_width(f32::INFINITY)
}

pub fn submit_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add_sized([ui.available_width(), 32.0], egui::Button::new(text))
}

/// Enter inside any of the (single line) text inputs submits the form.
pub fn enter_pressed(ui: &egui::Ui, fields: &[&egui::Response]) -> bool {
    fields.iter().any(|field| field.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter))
}

/// garde rule: the string must contain something other than whitespace.
pub fn not_blank(value: &str, _ctx: &()) -> garde::Result {
    if value.trim().is_empty() {
        return Err(garde::Error::new("must not be empty"));
    }
    Ok(())
}
