// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::asset::io::web::WebAssetPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{UpdateMode, WinitSettings, WINIT_WINDOWS};
use bevy::DefaultPlugins;
use bevy_diplomovka::GamePlugin;
use bevy_mod_reqwest::*;
use std::io::Cursor;
use std::time::Duration;
use winit::window::Icon; // ToDo: Replace bevy_game with your new crate name.
#[macro_use]
extern crate dotenv_codegen;

fn main() {
    // this breaks the wasm build
    // env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins(
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
                })
                // models are loaded over http(s) from the backend
                .set(WebAssetPlugin {
                    silence_startup_warning: true,
                }),
        )
        .add_systems(Startup, init_refresh_rate)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(ReqwestPlugin::default())
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}

fn init_refresh_rate(mut winit: ResMut<WinitSettings>) {
    winit.focused_mode = UpdateMode::reactive(Duration::from_secs_f32(1.0 / 30.0));
    winit.unfocused_mode = UpdateMode::reactive_low_power(Duration::from_secs(1));
}

// Sets the icon on windows and X11
fn set_window_icon(primary_window: Query<Entity, With<PrimaryWindow>>, _: NonSendMarker) {
    let Ok(primary_entity) = primary_window.single() else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) else {
        return;
    };
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height).unwrap();
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(primary) = windows.get_window(primary_entity) {
            primary.set_window_icon(Some(icon));
        }
    });
}
