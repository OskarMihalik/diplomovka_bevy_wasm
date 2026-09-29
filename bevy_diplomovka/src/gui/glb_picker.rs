//! Picking a .glb file from disk.
//!
//! Native: uses `bevy_file_dialog`, which opens the OS file dialog directly.
//! Web: `bevy_file_dialog` (via rfd) first shows its own "Ok" popup, because browsers
//! only open the file picker in response to a user click. Instead we open a hidden
//! `<input type="file">` right away: the egui click is handled a few milliseconds after
//! the real browser click, which is still within the browser's user activation window.
//!
//! Either way the picked file ends up as the single [`UploadedGlbFile`] entity.

use bevy::prelude::*;

use super::gui::UploadedGlbFile;

pub struct GlbPickerPlugin;

impl Plugin for GlbPickerPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        app.init_resource::<web::PickedGlbFiles>()
            .add_systems(Update, web::receive_picked_files);
        #[cfg(not(target_arch = "wasm32"))]
        let _ = app;
    }
}

/// Opens the file explorer for choosing a .glb file.
pub fn open_glb_picker(commands: &mut Commands) {
    #[cfg(target_arch = "wasm32")]
    commands.queue(web::open_file_input);

    #[cfg(not(target_arch = "wasm32"))]
    {
        use bevy_file_dialog::prelude::*;

        use super::gui::GlbFileContents;

        commands
            .dialog()
            .add_filter("Glb", &["glb"])
            .load_file::<GlbFileContents>();
    }
}

/// Stores the picked file, replacing a previously picked one.
pub fn set_uploaded_glb(
    commands: &mut Commands,
    previous: impl IntoIterator<Item = Entity>,
    file_name: String,
    contents: Vec<u8>,
) {
    bevy::log::info!("Loaded file {file_name}");
    for entity in previous {
        commands.entity(entity).despawn();
    }
    commands.spawn(UploadedGlbFile {
        file_name,
        contents,
    });
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::sync::{Arc, Mutex};

    use bevy::prelude::*;
    use wasm_bindgen::{prelude::Closure, JsCast};
    use wasm_bindgen_futures::JsFuture;

    use super::{set_uploaded_glb, UploadedGlbFile};

    /// Files read by the browser callbacks, waiting to be moved into the ECS.
    #[derive(Resource, Default, Clone)]
    pub struct PickedGlbFiles(Arc<Mutex<Vec<(String, Vec<u8>)>>>);

    pub fn open_file_input(world: &mut World) {
        let queue = world.resource::<PickedGlbFiles>().0.clone();
        if let Err(error) = try_open_file_input(queue) {
            bevy::log::error!("Could not open file picker: {error:?}");
        }
    }

    fn try_open_file_input(
        queue: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or("no document")?;
        let input: web_sys::HtmlInputElement = document.create_element("input")?.dyn_into()?;
        input.set_type("file");
        input.set_accept(".glb");

        let onchange_input = input.clone();
        let onchange = Closure::once_into_js(move || {
            let Some(file) = onchange_input.files().and_then(|files| files.get(0)) else {
                return;
            };
            wasm_bindgen_futures::spawn_local(async move {
                match JsFuture::from(file.array_buffer()).await {
                    Ok(buffer) => {
                        let contents = js_sys::Uint8Array::new(&buffer).to_vec();
                        queue.lock().unwrap().push((file.name(), contents));
                    }
                    Err(error) => bevy::log::error!("Could not read file: {error:?}"),
                }
            });
        });
        input.set_onchange(Some(onchange.unchecked_ref()));
        input.click();
        Ok(())
    }

    pub fn receive_picked_files(
        mut commands: Commands,
        picked: Res<PickedGlbFiles>,
        query_loaded_glb: Query<Entity, With<UploadedGlbFile>>,
    ) {
        let files = std::mem::take(&mut *picked.0.lock().unwrap());
        // only the last one matters, a newer file replaces the older
        if let Some((file_name, contents)) = files.into_iter().last() {
            set_uploaded_glb(&mut commands, query_loaded_glb.iter(), file_name, contents);
        }
    }
}
