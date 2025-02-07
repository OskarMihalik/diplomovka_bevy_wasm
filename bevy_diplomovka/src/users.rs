use bevy::prelude::*;
use dto::auth::UserDto;

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
