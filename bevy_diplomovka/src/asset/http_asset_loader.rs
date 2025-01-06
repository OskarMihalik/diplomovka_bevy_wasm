use bevy::prelude::*;

use crate::asset::source;

use super::source::http_source_plugin;

// #[cfg(target_family = "wasm")]

pub struct BlobLoaderPlugin;

impl Plugin for BlobLoaderPlugin {
    // #[allow(unused)]
    fn build(&self, app: &mut App) {
        // #[cfg(target_family = "wasm")]
        {
            // use bevy::asset::io::{AssetSource, AssetSourceId};
            // app.register_asset_source(
            //     "blob",
            //     AssetSource::build().with_reader(|| Box::new(source::)),
            // );
            app.add_plugins(http_source_plugin);
        }
    }
}
