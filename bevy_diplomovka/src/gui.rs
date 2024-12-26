use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_file_dialog::prelude::*;
pub struct GuiPlugin;
struct TextFileContents;

#[derive(Default, Resource)]
struct UiState {
    label: String,
    value: f32,
}

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
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
            .init_resource::<OccupiedScreenSpace>()
            .add_systems(Update, ui_example_system)
            .add_systems(
                Update,
                (
                    file_loaded,
                    file_saved,
                    file_load_canceled,
                    file_save_canceled,
                ),
            );
    }
}

fn ui_example_system(
    mut commands: Commands,
    mut is_last_selected: Local<bool>,
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
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

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
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
