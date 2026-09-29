use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

pub fn theme_picker(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Window::new("Theme picker")
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
        .show(ctx, |ui| egui::global_theme_preference_switch(ui));
}
