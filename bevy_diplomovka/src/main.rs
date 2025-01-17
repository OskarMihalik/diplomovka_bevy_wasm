// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod asset;
use asset::http_asset_loader::http_source_plugin;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{UpdateMode, WinitSettings, WinitWindows};
use bevy::DefaultPlugins;
use bevy_diplomovka::GamePlugin;
use bevy_mod_reqwest::*;
use std::io::Cursor;
use std::time::Duration;
use winit::window::Icon; // ToDo: Replace bevy_game with your new crate name.
                         // mod asset;

fn main() {
    App::new()
        .add_plugins((
            http_source_plugin,
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
                        fit_canvas_to_parent: true,
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        ))
        .add_systems(Startup, init_refresh_rate)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(ReqwestPlugin::default())
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}

fn init_refresh_rate(mut winit: ResMut<WinitSettings>) {
    winit.focused_mode = UpdateMode::reactive_low_power(Duration::from_secs_f32(1.0 / 30.0));
    winit.unfocused_mode = UpdateMode::reactive_low_power(Duration::from_secs_f32(1.0 / 30.0));
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
