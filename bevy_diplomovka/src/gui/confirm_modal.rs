use bevy_egui::egui::{self, Context, Id};

pub enum ConfirmModalResult {
    Confirm,
    Cancel,
    Nothing,
}

pub fn confirm_modal(ctx: &Context, title: &str, is_open: &bool) -> ConfirmModalResult {
    if !*is_open {
        return ConfirmModalResult::Nothing;
    }
    let mut result = ConfirmModalResult::Nothing;
    let modal = egui::Modal::new(Id::new(format!("Confirm modal {title}"))).show(ctx, |ui| {
        ui.label(title);
        ui.horizontal(|ui| {
            if ui.button("Confirm").clicked() {
                result = ConfirmModalResult::Confirm;
            }
            if ui.button("Cancel").clicked() {
                result = ConfirmModalResult::Cancel;
            }
        });
    });
    if modal.should_close() {
        result = ConfirmModalResult::Cancel;
    }

    return result;
}
