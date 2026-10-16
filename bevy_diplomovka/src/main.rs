// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::asset::io::web::WebAssetPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
#[cfg(feature = "debug_tools")]
use bevy::window::{PresentMode, WindowResolution};
#[cfg(not(feature = "debug_tools"))]
use bevy::winit::{UpdateMode, WinitSettings};
use bevy::winit::WINIT_WINDOWS;
use bevy::DefaultPlugins;
use bevy_diplomovka::GamePlugin;
use bevy_mod_reqwest::*;
use std::io::Cursor;
#[cfg(not(feature = "debug_tools"))]
use std::time::Duration;
use winit::window::Icon; // ToDo: Replace bevy_game with your new crate name.
#[macro_use]
extern crate dotenv_codegen;

fn main() {
    // this breaks the wasm build
    // env::set_var("RUST_BACKTRACE", "1");
    #[cfg(not(target_arch = "wasm32"))]
    bevy_diplomovka::system_info::init_local_offset();
    // bench mode for rendering_runner, see src/bench_mode.rs
    #[cfg(feature = "debug_tools")]
    let bench = match bevy_diplomovka::bench_mode::BenchConfig::from_env() {
        Ok(bench) => bench,
        Err(error) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                eprintln!("{error}");
                std::process::exit(2);
            }
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::error_1(&error.as_str().into());
                bevy_diplomovka::report::error(&error);
                Default::default()
            }
        }
    };

    #[allow(unused_mut)]
    let mut window = Window {
        title: "Bevy game".to_string(), // ToDo
        fit_canvas_to_parent: true,
        // Bind to canvas included in `index.html`
        canvas: Some("#bevy".to_owned()),
        // Tells wasm not to override default event handling, like F5 and Ctrl+R
        prevent_default_event_handling: false,
        ..default()
    };
    #[cfg(feature = "debug_tools")]
    if let Some((width, height)) = bench.resolution {
        // physical pixels, so every machine and browser renders the same amount, like
        // bevy_rendering_benchmark; uncapped, browsers still sync through requestAnimationFrame
        window.resolution = WindowResolution::new(width, height).with_scale_factor_override(1.0);
        window.fit_canvas_to_parent = false;
        window.resizable = false;
        window.present_mode = PresentMode::AutoNoVsync;
    }

    let mut app = App::new();
    app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(window),
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
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(ReqwestPlugin::default())
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon);
    #[cfg(feature = "debug_tools")]
    {
        app.insert_resource(bench);
        // after DefaultPlugins, whose PanicHandlerPlugin installs the hook that logs to the console
        #[cfg(target_arch = "wasm32")]
        {
            let log_panic = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                log_panic(info);
                bevy_diplomovka::report::error(&format!("panic: {info}"));
            }));
        }
    }
    // uncapped with debug_tools, so benchmarks measure the model and not the frame limiter
    #[cfg(not(feature = "debug_tools"))]
    app.add_systems(Startup, init_refresh_rate);
    app.run();
}

#[cfg(not(feature = "debug_tools"))]
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
