use std::process::Command;

use bevy::{prelude::*, state::commands};
use bevy_mod_reqwest::*;
use dto::{
    default::{NewTagDto, TagDto, TagDtoResponse},
    model::ModelDtoResponse,
};

use crate::{
    building::{ModelData, RebuildTagsEvent, SpawnModelEvent, TagData},
    gui::gui::{ShowErrorEvent, UiState},
};

const BACKEND_URL: &str = "http://localhost:4000";

pub struct ApiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_observer(get_tags)
            .add_observer(create_new_tag)
            .add_observer(get_model)
            .add_observer(get_tags)
            .add_observer(update_tag);
    }
}

#[derive(Event)]
pub struct GetTagsEvent {
    pub model_id: i32,
}

fn get_tags(trigger: Trigger<GetTagsEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/tags/{:?}", trigger.model_id);

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
             mut commands: Commands,
             query_models: Query<(Entity, &ModelData)>| {
                let parent_entity = query_models.get_single().unwrap().0;
                let response = trigger.event();
                let status = response.status();
                let data = response.as_str().unwrap();
                bevy::log::info!("response: {status}, data: {data}");
                let parsed: TagDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    Ok(tags) => {
                        commands.trigger(RebuildTagsEvent {
                            new_tag_dtos: tags,
                            parent_entity,
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
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

#[derive(Event)]
pub struct CreateNewTagEvent {
    pub new_tag_dto: NewTagDto,
    pub parent_entity: Entity,
}

fn create_new_tag(trigger: Trigger<CreateNewTagEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/tags");
    let body = trigger.new_tag_dto.clone();
    let parent_entity = trigger.parent_entity.clone();
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client.post(url).json(&body).build().unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let response = trigger.event();
                let status = response.status();
                let data = response.as_str().unwrap();
                bevy::log::info!("response: {status}, data: {data}");
                let parsed: TagDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    Ok(tags) => {
                        commands.trigger(RebuildTagsEvent {
                            new_tag_dtos: tags,
                            parent_entity,
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
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

#[derive(Event)]
pub struct GetModelEvent {
    pub model_id: i32,
}

fn get_model(trigger: Trigger<GetModelEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/model/{:?}", trigger.model_id);
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client.get(url).build().unwrap();

    client
        .send(reqwest_request)
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let response = trigger.event();
                let status = response.status();
                let data = response.as_str().unwrap();
                bevy::log::info!("response: {status}, data: {data}");
                let parsed: ModelDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    Ok(dto) => commands.trigger(SpawnModelEvent { model_dto: dto }),
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
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

#[derive(Event)]
pub struct UpdateTagEvent {
    pub tag_dto: TagDto,
    pub parent_entity: Entity,
}

fn update_tag(trigger: Trigger<UpdateTagEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/tags");
    let body = trigger.tag_dto.clone();
    let parent_entity = trigger.parent_entity.clone();
    let reqwest_request = client.patch(url).json(&body).build().unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let response = trigger.event();
                let status = response.status();
                let data = response.as_str().unwrap();
                bevy::log::info!("response: {status}, data: {data}");
                let parsed: TagDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    Ok(tags) => {
                        commands.trigger(RebuildTagsEvent {
                            new_tag_dtos: tags,
                            parent_entity,
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
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
