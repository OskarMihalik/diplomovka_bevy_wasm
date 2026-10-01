//! Web only: a model picked with the file input or dropped on the page is read by the browser
//! (index.html) and handed to Bevy through an in-memory asset source, so no server is needed.
//! index.html puts `{ name, bytes: Uint8Array }` into `window.benchModelFile`, it is taken from
//! there once. Reading the file from disk isn't part of the load time, "download + parsing" is
//! only parsing then.

use std::path::Path;

use bevy::asset::io::memory::{Dir, MemoryAssetReader};
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use wasm_bindgen::JsCast;

use crate::config::BenchConfig;
use crate::scene::BenchModel;
use crate::ModelAssetPath;

const SOURCE: &str = "memory";
const GLOBAL: &str = "benchModelFile";

/// The picked file, readable by the asset server as `memory://model.glb`
#[derive(Resource)]
struct PickedFiles(Dir);

/// Has to be added before `AssetPlugin`, asset sources are built there
pub struct FilePickerPlugin;

impl Plugin for FilePickerPlugin {
    fn build(&self, app: &mut App) {
        let dir = Dir::default();
        let reader_dir = dir.clone();
        app.register_asset_source(
            AssetSourceId::from(SOURCE),
            AssetSourceBuilder::new(move || {
                Box::new(MemoryAssetReader {
                    root: reader_dir.clone(),
                })
            }),
        )
        .insert_resource(PickedFiles(dir))
        // before Update, so scene.rs spawns the model in the same frame
        .add_systems(
            PreUpdate,
            take_picked_file.run_if(not(resource_exists::<ModelAssetPath>)),
        )
        .add_systems(
            Update,
            free_picked_file.run_if(resource_exists::<ModelAssetPath>),
        );
    }
}

fn take_picked_file(
    mut commands: Commands,
    files: Res<PickedFiles>,
    mut config: ResMut<BenchConfig>,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(file) = js_sys::Reflect::get(&window, &GLOBAL.into()) else {
        return;
    };
    if file.is_undefined() || file.is_null() {
        return;
    }
    // the bytes are copied into wasm memory, the page doesn't need to keep its copy
    let _ = js_sys::Reflect::delete_property(&window, &GLOBAL.into());

    let name = js_sys::Reflect::get(&file, &"name".into())
        .ok()
        .and_then(|name| name.as_string())
        .unwrap_or_default();
    let Ok(bytes) = js_sys::Reflect::get(&file, &"bytes".into())
        .and_then(|bytes| bytes.dyn_into::<js_sys::Uint8Array>())
    else {
        error!("window.{GLOBAL}.bytes must be a Uint8Array");
        return;
    };
    // id and name for the CSV come from the file name now, unless they are in the url
    match BenchConfig::with_picked_file(&name) {
        Ok(picked) => *config = picked,
        Err(error) => {
            error!("{error}");
            return;
        }
    }

    // a fixed name, `#` or `?` in the file name would break the asset path
    let extension = if name.to_lowercase().ends_with(".gltf") {
        "gltf"
    } else {
        "glb"
    };
    let path = format!("model.{extension}");
    info!(
        "model picked: {name}, {:.1} MiB",
        bytes.length() as f64 / (1024. * 1024.)
    );
    files.0.insert_asset(Path::new(&path), bytes.to_vec());
    commands.insert_resource(ModelAssetPath(format!("{SOURCE}://{path}")));
}

/// Once the model is parsed the file isn't needed anymore, a big model would be kept twice
fn free_picked_file(
    files: Res<PickedFiles>,
    model_path: Res<ModelAssetPath>,
    asset_server: Res<AssetServer>,
    q_model: Query<&WorldAssetRoot, With<BenchModel>>,
    mut freed: Local<bool>,
) {
    if *freed
        || !q_model
            .iter()
            .any(|root| asset_server.is_loaded_with_dependencies(&root.0))
    {
        return;
    }
    if let Some(path) = model_path.0.strip_prefix(&format!("{SOURCE}://")) {
        files.0.remove_asset(Path::new(path));
    }
    *freed = true;
}
