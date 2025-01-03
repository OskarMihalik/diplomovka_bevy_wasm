use bevy::prelude::*;
use iyes_perf_ui::{
    prelude::{PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiRoot},
    PerfUiPlugin,
};

use crate::GameState;

pub struct PerfUI;

impl Plugin for PerfUI {
    fn build(&self, app: &mut App) {
        app.add_plugins(PerfUiPlugin)
            .add_systems(OnEnter(GameState::ViewingModel), setup);
    }
}

fn setup(mut commands: Commands) {
    // // spawn a camera to be able to see anything
    // commands.spawn(Camera2d);

    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}
