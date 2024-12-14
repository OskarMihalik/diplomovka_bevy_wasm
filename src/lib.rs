#![allow(clippy::type_complexity)]

mod api;
mod building;
mod loading;
mod menu;

use crate::loading::LoadingPlugin;

use api::ApiPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_flycam::prelude::*;
use building::BuildingPlugin;
// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ApiPlugin)
            .init_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                BuildingPlugin,
                PlayerPlugin,
                MeshPickingPlugin,
            ));

        // #[cfg(debug_assertions)]
        // {
        //     app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        // }
    }
}
