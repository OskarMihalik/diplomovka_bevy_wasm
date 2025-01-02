use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts, EguiPlugin,
};
use bevy_file_dialog::prelude::*;
use dto::default::TagDto;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use crate::building::TagData;
pub struct GuiPlugin;
struct TextFileContents;

#[derive(Default, Resource)]
pub struct UiState {}

#[derive(Default, Resource)]
pub struct UiContexts {
    pub toasts: Toasts,
}

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(
                FileDialogPlugin::new()
                    // allow saving of files marked with TextFileContents
                    .with_save_file::<TextFileContents>()
                    // allow loading of files marked with TextFileContents
                    .with_load_file::<TextFileContents>(),
            )
            .init_resource::<UiState>()
            .init_resource::<UiContexts>()
            .add_systems(Startup, setup)
            .add_systems(Update, ui_example_system)
            .add_systems(
                Update,
                (
                    file_loaded,
                    file_saved,
                    file_load_canceled,
                    file_save_canceled,
                ),
            )
            .add_observer(show_error);
    }
}

fn setup(mut ui_context: ResMut<UiContexts>) {
    ui_context.toasts = Toasts::new()
        .anchor(Align2::RIGHT_TOP, (-10.0, -10.0)) // 10 units from the bottom right corner
        .direction(egui::Direction::TopDown);
}

fn ui_example_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    ui_state: Res<UiState>,
    mut ui_contexts: ResMut<UiContexts>,
    query_tags: Query<&TagData>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Some things");
            if ui.add(egui::widgets::Button::new("Load model")).clicked() {
                commands
                    .dialog()
                    .add_filter("Text", &["txt"])
                    .load_file::<TextFileContents>();
            }

            if ui.add(egui::widgets::Button::new("Save model")).clicked() {
                commands
                    .dialog()
                    .add_filter("Text", &["txt"])
                    .set_file_name("hello.txt")
                    .save_file::<TextFileContents>(b"hello".to_vec());
            };
            ui.heading("Tags");
            ui.separator();
            ui.vertical(|ui| {
                for tag in query_tags.iter() {
                    ui.label(format!("{}: {}", tag.dto.id, tag.dto.title));
                }
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    ui_contexts.toasts.show(ctx);
}

#[derive(Event)]
pub struct ShowErrorEvent {
    pub message: String,
}

fn show_error(
    trigger: Trigger<ShowErrorEvent>,
    mut ui_contexts: ResMut<UiContexts>,
    mut contexts: EguiContexts,
) {
    let message = &trigger.event().message;
    let ctx = contexts.ctx_mut();
    bevy::log::error!("Error: {}", message);
    ui_contexts.toasts.add(Toast {
        text: message.into(),
        kind: ToastKind::Error,
        options: ToastOptions::default()
            .duration_in_seconds(5.0)
            .show_progress(true),
        ..Default::default()
    });
}

fn file_loaded(mut ev_loaded: EventReader<DialogFileLoaded<TextFileContents>>) {
    for ev in ev_loaded.read() {
        bevy::log::info!(
            "Loaded file {} with contents '{}'",
            ev.file_name,
            std::str::from_utf8(&ev.contents).unwrap()
        );
    }
}

fn file_load_canceled(mut ev_canceled: EventReader<DialogFileLoadCanceled<TextFileContents>>) {
    for _ in ev_canceled.read() {
        bevy::log::info!("Text file content load canceled");
    }
}

fn file_saved(mut ev_saved: EventReader<DialogFileSaved<TextFileContents>>) {
    for ev in ev_saved.read() {
        match ev.result {
            Ok(_) => bevy::log::info!("File {} successfully saved", ev.file_name),
            Err(ref err) => bevy::log::info!("Failed to save {}: {}", ev.file_name, err),
        }
    }
}

fn file_save_canceled(mut ev_canceled: EventReader<DialogFileSaveCanceled<TextFileContents>>) {
    for _ in ev_canceled.read() {
        bevy::log::info!("Text file content save canceled");
    }
}
