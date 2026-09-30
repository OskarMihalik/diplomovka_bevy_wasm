//! Rendering benchmark: opens one glb model the way bevy_diplomovka does and measures how long
//! it takes to load and how fast it renders while the camera orbits it. See README.md.

mod bench;
mod config;
mod load_timing;
mod measure;
mod scene;
mod system_info;

use bevy::asset::io::web::WebAssetPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::{PresentMode, WindowResolution};
use bevy::winit::WinitSettings;
use config::BenchConfig;

/// What to pass to `AssetServer::load` for the model scene
#[derive(Resource)]
pub struct ModelAssetPath(pub String);

fn main() {
    let config = match BenchConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&error.into());
            #[cfg(not(target_arch = "wasm32"))]
            {
                eprintln!("{error}");
                std::process::exit(2);
            }
            #[allow(unreachable_code)]
            return;
        }
    };

    // a local file is loaded from its own folder, which becomes the asset root
    let (asset_root, model_path) = if config.is_url() {
        ("assets".to_string(), config.model.clone())
    } else {
        local_model_path(&config.model)
    };

    let mut wgpu = WgpuSettings::default();
    if let Some(power_preference) = config.power_preference {
        wgpu.power_preference = power_preference;
    }

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy rendering benchmark".to_string(),
                    // physical pixels, so every machine and browser renders the same amount
                    resolution: WindowResolution::new(config.width, config.height)
                        .with_scale_factor_override(1.0),
                    // uncapped, browsers still sync to the display through requestAnimationFrame
                    present_mode: PresentMode::AutoNoVsync,
                    resizable: false,
                    canvas: Some("#bevy".to_owned()),
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(wgpu)),
                ..default()
            })
            .set(AssetPlugin {
                file_path: asset_root,
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WebAssetPlugin {
                silence_startup_warning: true,
            }),
    )
    .insert_resource(WinitSettings::continuous())
    .insert_resource(ModelAssetPath(model_path))
    .insert_resource(config)
    .add_plugins((
        scene::ScenePlugin,
        load_timing::LoadTimingPlugin,
        bench::BenchPlugin,
    ));
    app.run();
}

/// `(folder, file name)` of a model on disk
#[cfg(not(target_arch = "wasm32"))]
fn local_model_path(model: &str) -> (String, String) {
    let path = std::path::Path::new(model);
    let path = path.canonicalize().unwrap_or_else(|error| {
        eprintln!("can't open model `{model}`: {error}");
        std::process::exit(2);
    });
    let folder = path.parent().unwrap_or(std::path::Path::new("/"));
    let file = path.file_name().unwrap_or_default();
    (
        folder.to_string_lossy().into_owned(),
        file.to_string_lossy().into_owned(),
    )
}

/// On the web `model` was already resolved to an absolute url against the page, see config.rs
#[cfg(target_arch = "wasm32")]
fn local_model_path(model: &str) -> (String, String) {
    ("assets".to_string(), model.to_string())
}
