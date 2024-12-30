use std::process::Command;

use bevy::prelude::*;
use bevy_mod_reqwest::*;
use dto::default::{DtoResponse, TagDto, TagDtoResponse, Test};

use crate::gui::gui::{ShowErrorEvent, UiState};

pub struct ApiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(get_tags);
    }
}

#[derive(Event)]
pub struct GetTagsEvent {}

fn get_tags(_trigger: Trigger<GetTagsEvent>, mut client: BevyReqwest) {
    let url = "http://localhost:4000/tags";

    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client.get(url).build().unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut ui_state: ResMut<UiState>,
             mut commands: Commands| {
                let response = trigger.event();
                let status = response.status();
                let data = response.as_str().unwrap();
                bevy::log::info!("response: {status}, data: {data}");
                let parsed: TagDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    DtoResponse::Ok(tags) => {
                        ui_state.tags = tags;
                    }
                    DtoResponse::Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            bevy::log::info!("error: {e:?}");
        });
}
