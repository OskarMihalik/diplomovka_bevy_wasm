#![allow(clippy::type_complexity)]

mod api;
mod asset;
mod building;
mod gui;
mod loading;
mod utils;

use crate::loading::LoadingPlugin;

use api::ApiPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use building::BuildingPlugin;
use dto::project::ProjectDto;
use gui::{gui::GuiPlugin, perf_ui::PerfUI};
use utils::log_entity_components;
// use gui::GuiPlugin;
// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    InitialLoading,
    SelectingProjectAndModel,
    // During this State the actual game logic is executed
    ViewingModel,
}

#[derive(Resource)]
pub struct ProjectDtoRes {
    pub project_dto: ProjectDto,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ApiPlugin)
            .init_state::<GameState>()
            .add_plugins((
                PanOrbitCameraPlugin,
                LoadingPlugin,
                BuildingPlugin,
                MeshPickingPlugin,
                GuiPlugin,
                PerfUI,
            ))
            .add_observer(log_entity_components);

        // #[cfg(debug_assertions)]
        // {
        //     app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        // }
    }
}
