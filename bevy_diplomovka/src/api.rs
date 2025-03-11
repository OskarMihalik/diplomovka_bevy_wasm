use std::collections::HashMap;

use ::serde::de;
use bevy::prelude::*;
use bevy_mod_reqwest::*;
use dto::{
    auth::{AuthDtoResponse, LoginDto, RegisterDto},
    default::{
        CreatedTagMessageDtoResponse, EmptyResponse, NewStatusDto, NewTagDto, NewTagMessageDto,
        StatusDto, StatusesResponse, TagDto, TagDtoResponse, TagMessageDto, TagMessagesDtoResponse,
    },
    model::{ModelDto, ModelDtoResponse, ModelsDtoResponse},
    project::{NewProjectDto, ProjectDtoResponse, ProjectsDtoResponse},
    users::{
        GetUsersDto, OtherUserDtoResponse, OtherUsersDtoResponse, ProjectUsersDtoResponse,
        UserToProjectDto,
    },
};

use crate::{
    building::{
        ModelData, ProjectData, ProjectStatusesData, RebuildTagsEvent, TagData, TagMessagesData,
        ThisModelIsSelected, ThisProjectIsSelected,
    },
    gui::gui::ShowErrorEvent,
    models::UpdateModelsEvent,
    users::{LoggedUser, OtherUsers, UsersInProject},
    utils::get_token_from_user,
    GameState,
};

pub const BACKEND_URL: &str = "http://localhost:4000";

pub struct ApiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(get_tags)
            .add_observer(create_new_tag)
            .add_observer(get_model)
            .add_observer(get_tags)
            .add_observer(get_project)
            .add_observer(update_tag)
            .add_observer(get_models)
            .add_observer(create_model)
            .add_observer(login_user)
            .add_observer(register_user)
            .add_observer(insert_project)
            .add_observer(get_users)
            .add_observer(get_users_in_project)
            .add_observer(add_user_to_project)
            .add_observer(get_tag_messages)
            .add_observer(create_tag_message)
            .add_observer(get_statuses)
            .add_observer(update_status)
            .add_observer(create_status)
            .add_observer(delete_status)
            .add_observer(get_projects);
    }
}

pub fn parse_response<T>(event: Trigger<'_, ReqwestResponseEvent>) -> T
where
    T: de::DeserializeOwned,
{
    let response = event.event();
    let data = response.as_str().unwrap();
    println!("data: {:?}", data);
    let parsed: T = serde_json::from_str(data).unwrap();
    return parsed;
}

pub fn on_reqwest_error(trigger: Trigger<ReqwestErrorEvent>, mut commands: Commands) {
    let e = &trigger.event().0;
    bevy::log::error!("Error: {:?}", &trigger.event());
    commands.trigger(ShowErrorEvent {
        message: e.to_string(),
    })
}

#[derive(Event)]
pub struct GetTagsEvent {
    pub model_id: i32,
}

fn get_tags(
    trigger: Trigger<GetTagsEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tags/{:?}", trigger.model_id);

    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut commands: Commands,
             query_model: Single<(Entity, &ModelData, &ThisModelIsSelected)>| {
                let parent_entity = query_model.0;
                let parsed = parse_response::<TagDtoResponse>(trigger);
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
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct CreateNewTagEvent {
    pub new_tag_dto: NewTagDto,
    pub parent_entity: Entity,
}

fn create_new_tag(
    trigger: Trigger<CreateNewTagEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tags");
    let body = trigger.new_tag_dto.clone();
    let parent_entity = trigger.parent_entity.clone();
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .post(url)
        .json(&body)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed: TagDtoResponse = parse_response(trigger);
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
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetModelEvent {
    pub model_id: i32,
}

fn get_model(
    trigger: Trigger<GetModelEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/model/{:?}", trigger.model_id);
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed: ModelDtoResponse = parse_response(trigger);
                match parsed {
                    Ok(_) => (),
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct CreateModelEvent {
    pub name: String,
    pub project_id: i32,
    pub model_bytes: Vec<u8>,
}

fn create_model(
    trigger: Trigger<CreateModelEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!(
        "{BACKEND_URL}/model/{}/{}",
        trigger.project_id, trigger.name
    );
    let multipart = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(trigger.model_bytes.clone()),
    );
    // let parent_entity = trigger.parent_entity.clone();
    let reqwest_request = client
        .post(url)
        .header("authorization", get_token_from_user(query_user))
        .multipart(multipart)
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed: ModelsDtoResponse = parse_response(trigger);
                let dtos = match parsed {
                    Ok(dto) => dto,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
                commands.trigger(UpdateModelsEvent { dtos });
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetModelsEvent {
    pub project_id: i32,
}

fn get_models(
    trigger: Trigger<GetModelsEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/models/{:?}", trigger.project_id);
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed: ModelsDtoResponse = parse_response(trigger);
                let dtos = match parsed {
                    Ok(dto) => dto,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
                commands.trigger(UpdateModelsEvent { dtos });
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct UpdateTagEvent {
    pub tag_dto: TagDto,
    pub parent_entity: Entity,
}

fn update_tag(
    trigger: Trigger<UpdateTagEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tags");
    let body = trigger.tag_dto.clone();
    let parent_entity = trigger.parent_entity.clone();
    let reqwest_request = client
        .patch(url)
        .json(&body)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed: TagDtoResponse = parse_response(trigger);
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
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetProjectEvent {
    pub project_id: i32,
}

fn get_project(
    trigger: Trigger<GetProjectEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/project/{:?}", trigger.project_id);

    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let response = trigger.event();
                let data = response.as_str().unwrap();
                let parsed: ProjectDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    Ok(dto) => {
                        // TODO: update selected project
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetProjectsEvent {}

fn get_projects(
    _trigger: Trigger<GetProjectsEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url: String = format!("{BACKEND_URL}/project");

    // use regular reqwest http calls, then poll them to completion.
    bevy::log::info!("sending request to {url}");
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();
    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut commands: Commands,
             query_selected_project: Option<
                Single<(Entity, &ProjectData, &ThisProjectIsSelected)>,
            >,
             query_projects: Query<(Entity, &ProjectData)>| {
                let response = trigger.event();
                let data = response.as_str().unwrap();
                let parsed: ProjectsDtoResponse = serde_json::from_str(data).unwrap();
                let dtos = match parsed {
                    Ok(dtos) => dtos,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
                match query_selected_project {
                    Some(selected) => {
                        for (entity, _) in query_projects.iter() {
                            commands.entity(entity).despawn_recursive()
                        }
                        for dto in dtos {
                            let dto_id = dto.id;
                            let mut builder = commands.spawn(ProjectData { dto });
                            if dto_id == selected.1.dto.id {
                                builder.insert(ThisProjectIsSelected {});
                            }
                        }
                    }
                    None => {
                        for (entity, _) in query_projects.iter() {
                            commands.entity(entity).despawn_recursive()
                        }
                        for dto in dtos {
                            commands.spawn(ProjectData { dto });
                        }
                    }
                }
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct NewProjectEvent {
    pub dto: NewProjectDto,
}

fn insert_project(
    trigger: Trigger<NewProjectEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url: String = format!("{BACKEND_URL}/project");
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();
    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ProjectsDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(_) => commands.trigger(GetProjectsEvent {}),
                    Err(error) => commands.trigger(ShowErrorEvent {
                        message: error.message.clone(),
                    }),
                }
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct LoginEvent {
    pub dto: LoginDto,
}

fn login_user(trigger: Trigger<LoginEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/login");

    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client.post(url).json(&trigger.dto).build().unwrap();

    client
        .send(reqwest_request)
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut commands: Commands,
             query_user: Option<Single<(Entity, &LoggedUser)>>| {
                let parsed = parse_response::<AuthDtoResponse>(trigger);
                match parsed {
                    Ok(user_dto) => {
                        if let Some(query_user) = query_user {
                            commands.entity(query_user.0).despawn_recursive();
                        }
                        commands.spawn(LoggedUser { dto: user_dto });
                        commands.set_state(GameState::SelectingProjectAndModel);
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

// todo register
#[derive(Event)]
pub struct RegisterEvent {
    pub dto: RegisterDto,
}
// todo
fn register_user(trigger: Trigger<RegisterEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/register");

    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client.post(url).json(&trigger.dto).build().unwrap();

    client
        .send(reqwest_request)
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut commands: Commands,
             query_user: Option<Single<(Entity, &LoggedUser)>>| {
                let parsed = parse_response::<AuthDtoResponse>(trigger);
                match parsed {
                    Ok(user_dto) => {
                        if let Some(query_user) = query_user {
                            commands.entity(query_user.0).despawn_recursive();
                        }
                        commands.spawn(LoggedUser { dto: user_dto });
                        commands.set_state(GameState::SelectingProjectAndModel);
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetUsersEvent {
    pub dto: GetUsersDto,
}
fn get_users(
    trigger: Trigger<GetUsersEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/users");
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut commands: Commands,
                  query_user: Option<Single<(Entity, &OtherUsers)>>| {
                let parsed: OtherUsersDtoResponse = parse_response(trigger);
                match parsed {
                    Ok(dtos) => {
                        if let Some(query_user) = query_user {
                            commands.entity(query_user.0).despawn_recursive();
                        }
                        commands.spawn(OtherUsers { dtos });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetUsersInProjectEvent {
    pub project_id: i32,
}
fn get_users_in_project(
    trigger: Trigger<GetUsersInProjectEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/users/{}", trigger.project_id);
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut commands: Commands,
                  query_user: Option<Single<(Entity, &UsersInProject)>>| {
                let parsed: ProjectUsersDtoResponse = parse_response(trigger);
                match parsed {
                    Ok(dtos) => {
                        if let Some(query_user) = query_user {
                            commands.entity(query_user.0).despawn_recursive();
                        }
                        commands.spawn(UsersInProject { dtos });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct AddUserToProjectEvent {
    pub dto: UserToProjectDto,
}
fn add_user_to_project(
    trigger: Trigger<AddUserToProjectEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/project_user");
    // use regular reqwest http calls, then poll them to completion.
    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut commands: Commands,
                  selected_project_query: Option<
                Single<(&ProjectData, &ThisProjectIsSelected)>,
            >| {
                let parsed: OtherUserDtoResponse = parse_response(trigger);
                match parsed {
                    Ok(dto) => dto,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
                if let Some(selected_project_query) = selected_project_query {
                    commands.trigger(GetUsersInProjectEvent {
                        project_id: selected_project_query.0.dto.id,
                    });
                }
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct CreateTagMessageEvent {
    pub dto: NewTagMessageDto,
}

fn create_tag_message(
    trigger: Trigger<CreateTagMessageEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tag_message");
    let body = trigger.dto.clone();
    let reqwest_request = client
        .post(url)
        .json(&body)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut commands: Commands,
                  query_tags: Query<(Entity, &TagData, Option<&TagMessagesData>)>| {
                let parsed: CreatedTagMessageDtoResponse = parse_response(trigger);
                bevy::log::info!("CreatedTagMessageDtoResponse: {:?}", parsed);
                match parsed {
                    Ok(_) => {
                        commands.trigger(GetTagMessagesEvent {
                            tag_id: body.tag_id,
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message,
                    }),
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetTagMessagesEvent {
    pub tag_id: i32,
}

fn get_tag_messages(
    trigger: Trigger<GetTagMessagesEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tag_message/{:?}", trigger.tag_id);
    let tag_id = trigger.tag_id;
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut commands: Commands,
                  mut query_tags: Query<(Entity, &TagData, Option<&mut TagMessagesData>)>| {
                let parsed = match parse_response::<TagMessagesDtoResponse>(trigger) {
                    Ok(ok) => ok,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };

                let tag = match query_tags.iter_mut().find(|query| query.1.dto.id == tag_id) {
                    Some(ok) => ok,
                    None => {
                        return;
                    }
                };

                match tag.2 {
                    Some(mut tag_message) => {
                        tag_message.dtos.clear();
                        tag_message.dtos.extend(parsed);
                        bevy::log::info!("tag_message.dtos: {:?}", tag_message.dtos);
                    }
                    None => {
                        if let Some(mut entity) = commands.get_entity(tag.0) {
                            entity.insert(TagMessagesData { dtos: parsed.clone() });
                            bevy::log::info!("TagMessagesData: {:?}", parsed);

                        }
                    }
                }
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct GetStatusesEvent {
    pub project_id: i32,
}

fn get_statuses(
    trigger: Trigger<GetStatusesEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/status/{:?}", trigger.project_id);
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut commands: Commands,
                  query_status: Query<(Entity, &ProjectStatusesData)>| {
                let parsed = match parse_response::<StatusesResponse>(trigger) {
                    Ok(ok) => ok,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };

                query_status.iter().for_each(|(entity, _status)| {
                    commands.entity(entity).despawn_recursive();
                });

                commands.spawn(ProjectStatusesData { dtos: parsed });
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct UpdateStatusEvent {
    pub dto: StatusDto,
}

fn update_status(
    trigger: Trigger<UpdateStatusEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/status");
    let body = trigger.dto.clone();
    let reqwest_request = client
        .post(url)
        .json(&body)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed = match parse_response::<EmptyResponse>(trigger) {
                    Ok(ok) => {
                        commands.trigger(GetStatusesEvent {
                            project_id: body.project_id,
                        });
                    }
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct CreateStatusEvent {
    pub dto: NewStatusDto,
}

fn create_status(
    trigger: Trigger<CreateStatusEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/status");
    let body = trigger.dto.clone();
    let reqwest_request = client
        .put(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let parsed = match parse_response::<EmptyResponse>(trigger) {
                    Ok(ok) => {
                        commands.trigger(GetStatusesEvent {
                            project_id: body.project_id,
                        });
                    }
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct DeleteStatusEvent {
    pub status_id: i32,
    pub project_id: i32,
}

fn delete_status(
    trigger: Trigger<DeleteStatusEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/status/{:?}", trigger.status_id);
    let project_id = trigger.project_id.clone();
    let reqwest_request = client
        .delete(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        // Sends the created http request
        .send(reqwest_request)
        // The response from the http request can be reached using an observersystem,
        // where the only requirement is that the first parameter in the system is the specific Trigger type
        // the rest is the same as a regular system
        .on_response(
            move |trigger: Trigger<ReqwestResponseEvent>, mut commands: Commands| {
                let _ = match parse_response::<EmptyResponse>(trigger) {
                    Ok(_) => {
                        commands.trigger(GetStatusesEvent { project_id });
                    }
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message,
                        });
                        return;
                    }
                };
            },
        )
        // In case of request error, it can be reached using an observersystem as well
        .on_error(on_reqwest_error);
}
