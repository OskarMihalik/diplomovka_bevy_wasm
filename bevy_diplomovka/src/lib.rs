#![allow(clippy::type_complexity)]

mod api;
mod api_tracking;
mod building;
mod gui;
mod loading;
mod models;
mod users;
mod utils;

use crate::loading::LoadingPlugin;

use api::ApiPlugin;
use bevy::app::App;
use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use building::BuildingPlugin;
use gui::gui::GuiPlugin;
use models::ModelManagmentPlugin;
use utils::log_entity_components;
// use gui::GuiPlugin;
// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum GameState {
    Auth,
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    InitialLoading,
    SelectingProjectAndModel,
    // During this State the actual game logic is executed
    ViewingModel,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ApiPlugin)
            .init_state::<GameState>()
            .register_type::<GameState>()
            .add_plugins((
                PanOrbitCameraPlugin,
                LoadingPlugin,
                BuildingPlugin,
                MeshPickingPlugin,
                GuiPlugin,
                ModelManagmentPlugin,
                OutlinePlugin::JUMP_FLOOD,
            ))
            .add_observer(log_entity_components);

        // #[cfg(debug_assertions)]
        // {
        //     app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        // }
    }
}
