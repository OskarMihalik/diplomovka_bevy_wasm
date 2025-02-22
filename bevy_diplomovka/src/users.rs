use bevy::prelude::*;
use dto::{
    auth::UserDto,
    users::{OtherUserDto, ProjectUserDto},
};

pub struct UsersPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for UsersPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

#[derive(Component)]
pub struct LoggedUser {
    pub dto: UserDto,
}

#[derive(Component)]
pub struct OtherUsers {
    pub dtos: Vec<OtherUserDto>,
}

#[derive(Component)]
pub struct UsersInProject {
    pub dtos: Vec<ProjectUserDto>,
}
