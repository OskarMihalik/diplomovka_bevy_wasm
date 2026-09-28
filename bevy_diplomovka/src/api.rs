use ::serde::de;
use bevy::prelude::*;
use bevy_mod_reqwest::*;
use dto::{
    auth::{AuthDtoResponse, LoginDto, RegisterDto},
    default::{
        CreatedTagMessageDtoResponse, EmptyResponse, NewStatusDto, NewTagDto, NewTagMessageDto,
        StatusDto, StatusesResponse, TagDto, TagDtoResponse, TagMessagesDtoResponse,
    },
    model::{ModelDtoResponse, ModelsDtoResponse},
    project::{NewProjectDto, ProjectDto, ProjectDtoResponse, ProjectsDtoResponse},
    users::{
        GetUsersDto, OtherUserDtoResponse, OtherUsersDtoResponse, ProjectUsersDtoResponse,
        UserToProjectDto,
    },
};

use crate::{
    api_tracking::{send_tracked, TrackApiExt},
    building::{
        ProjectData, ProjectStatusesData, RebuildTagsEvent, TagData, TagMessagesData,
        ThisProjectIsSelected,
    },
    gui::gui::{ShowErrorEvent, ShowSuccessEvent},
    models::UpdateModelsEvent,
    users::{LoggedUser, OtherUsers, UsersInProject},
    utils::get_token_from_user,
    GameState,
};
use dotenv_codegen::dotenv;

pub const BACKEND_URL: &str = dotenv!("FRONTEND_BACKEND_URL");

pub struct ApiPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(get_tags)
            .add_observer(create_new_tag)
            .add_observer(get_model)
            .add_observer(get_project)
            .add_observer(update_tag)
            .add_observer(get_models)
            .add_observer(create_model)
            .add_observer(login_user)
            .add_observer(register_user)
            .add_observer(insert_project)
            .add_observer(delete_project)
            .add_observer(update_project)
            .add_observer(get_users)
            .add_observer(get_users_in_project)
            .add_observer(add_user_to_project)
            .add_observer(get_tag_messages)
            .add_observer(create_tag_message)
            .add_observer(get_statuses)
            .add_observer(update_status)
            .add_observer(create_status)
            .add_observer(delete_status)
            .add_observer(delete_tag)
            .add_observer(get_projects)
            .track_api::<NewProjectEvent>()
            .track_api::<CreateModelEvent>();
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
    show_reqwest_error(&mut commands, trigger.event());
}

/// Logs a request error and shows it as a toast.
/// Use in a custom `on_error` handler that also needs to do something else.
pub fn show_reqwest_error(commands: &mut Commands, error: &ReqwestErrorEvent) {
    bevy::log::error!("Error: {:?}", error);
    commands.trigger(ShowErrorEvent {
        message: error.0.to_string(),
    })
}

#[derive(Event)]
pub struct GetTagsEvent {
    pub project_id: i32,
}

fn get_tags(
    trigger: Trigger<GetTagsEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tags/{:?}", trigger.project_id);

    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<TagDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(tags) => {
                        commands.trigger(RebuildTagsEvent {
                            new_tag_dtos: tags.clone(),
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct DeleteTagEvent {
    pub tag_id: i32,
}

fn delete_tag(
    trigger: Trigger<DeleteTagEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tag/{:?}", trigger.tag_id);

    let reqwest_request = client
        .delete(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<EmptyResponse>>,
             mut commands: Commands,
             query_project: Single<(&ProjectData, &ThisProjectIsSelected)>| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetTagsEvent {
                            project_id: query_project.0.dto.id,
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    let reqwest_request = client
        .post(url)
        .json(&trigger.new_tag_dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<TagDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(tags) => {
                        commands.trigger(RebuildTagsEvent {
                            new_tag_dtos: tags.clone(),
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ModelDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(_) => (),
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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
    mut commands: Commands,
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

    let reqwest_request = client
        .post(url)
        .header("authorization", get_token_from_user(query_user))
        .multipart(multipart)
        .build()
        .unwrap();

    send_tracked::<CreateModelEvent>(&mut client, &mut commands, reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ModelsDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(dtos) => commands.trigger(UpdateModelsEvent { dtos: dtos.clone() }),
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ModelsDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(dtos) => commands.trigger(UpdateModelsEvent { dtos: dtos.clone() }),
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct UpdateTagEvent {
    pub tag_dto: TagDto,
}

fn update_tag(
    trigger: Trigger<UpdateTagEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/tags");

    let reqwest_request = client
        .patch(url)
        .json(&trigger.tag_dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<TagDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(tags) => {
                        commands.trigger(RebuildTagsEvent {
                            new_tag_dtos: tags.clone(),
                        });
                        commands.trigger(ShowSuccessEvent {
                            message: "Tag updated".to_string(),
                        });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ProjectDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(_dto) => {
                        // TODO: update selected project
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct UpdateProjectEvent {
    pub dto: ProjectDto,
}

fn update_project(
    trigger: Trigger<UpdateProjectEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/project");

    let reqwest_request = client
        .patch(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<EmptyResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetProjectsEvent {});
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct DeleteProjectEvent {
    pub project_id: i32,
}

fn delete_project(
    trigger: Trigger<DeleteProjectEvent>,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url = format!("{BACKEND_URL}/project/{}", trigger.project_id);

    let reqwest_request = client
        .delete(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<EmptyResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetProjectsEvent {});
                        commands.trigger(UpdateModelsEvent { dtos: vec![] });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    bevy::log::info!("sending request to {url}");
    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ProjectsDtoResponse>>,
             mut commands: Commands,
             query_selected_project: Option<Single<(&ProjectData, &ThisProjectIsSelected)>>,
             query_projects: Query<Entity, With<ProjectData>>| {
                let dtos = match &trigger.0 {
                    Ok(dtos) => dtos,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message.clone(),
                        });
                        return;
                    }
                };

                let selected_id = query_selected_project.map(|selected| selected.0.dto.id);

                for entity in query_projects.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for dto in dtos {
                    let mut builder = commands.spawn(ProjectData { dto: dto.clone() });
                    if Some(dto.id) == selected_id {
                        builder.insert(ThisProjectIsSelected {});
                    }
                }
            },
        )
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct NewProjectEvent {
    pub dto: NewProjectDto,
}

fn insert_project(
    trigger: Trigger<NewProjectEvent>,
    mut commands: Commands,
    mut client: BevyReqwest,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    let url: String = format!("{BACKEND_URL}/project");

    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    send_tracked::<NewProjectEvent>(&mut client, &mut commands, reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ProjectsDtoResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(_) => commands.trigger(GetProjectsEvent {}),
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}

/// Shared response handler for login and register: replaces the logged in user and moves on.
fn on_auth_response(
    trigger: Trigger<JsonResponse<AuthDtoResponse>>,
    mut commands: Commands,
    query_user: Option<Single<(Entity, &LoggedUser)>>,
) {
    match &trigger.0 {
        Ok(user_dto) => {
            if let Some(query_user) = query_user {
                commands.entity(query_user.0).despawn_recursive();
            }
            commands.spawn(LoggedUser {
                dto: user_dto.clone(),
            });
            commands.set_state(GameState::SelectingProjectAndModel);
        }
        Err(error_dto) => commands.trigger(ShowErrorEvent {
            message: error_dto.message.clone(),
        }),
    };
}

#[derive(Event)]
pub struct LoginEvent {
    pub dto: LoginDto,
}

fn login_user(trigger: Trigger<LoginEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/login");

    let reqwest_request = client.post(url).json(&trigger.dto).build().unwrap();

    client
        .send(reqwest_request)
        .on_json_response(on_auth_response)
        .on_error(on_reqwest_error);
}

#[derive(Event)]
pub struct RegisterEvent {
    pub dto: RegisterDto,
}

fn register_user(trigger: Trigger<RegisterEvent>, mut client: BevyReqwest) {
    let url = format!("{BACKEND_URL}/register");

    let reqwest_request = client.post(url).json(&trigger.dto).build().unwrap();

    client
        .send(reqwest_request)
        .on_json_response(on_auth_response)
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

    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<OtherUsersDtoResponse>>,
             mut commands: Commands,
             query_users: Option<Single<(Entity, &OtherUsers)>>| {
                match &trigger.0 {
                    Ok(dtos) => {
                        if let Some(query_users) = query_users {
                            commands.entity(query_users.0).despawn_recursive();
                        }
                        commands.spawn(OtherUsers { dtos: dtos.clone() });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<ProjectUsersDtoResponse>>,
             mut commands: Commands,
             query_users: Option<Single<(Entity, &UsersInProject)>>| {
                match &trigger.0 {
                    Ok(dtos) => {
                        if let Some(query_users) = query_users {
                            commands.entity(query_users.0).despawn_recursive();
                        }
                        commands.spawn(UsersInProject { dtos: dtos.clone() });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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

    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<OtherUserDtoResponse>>,
             mut commands: Commands,
             selected_project_query: Option<Single<(&ProjectData, &ThisProjectIsSelected)>>| {
                match &trigger.0 {
                    Ok(_) => {
                        if let Some(selected_project_query) = selected_project_query {
                            commands.trigger(GetUsersInProjectEvent {
                                project_id: selected_project_query.0.dto.id,
                            });
                        }
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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
    let tag_id = trigger.dto.tag_id;

    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            move |trigger: Trigger<JsonResponse<CreatedTagMessageDtoResponse>>,
                  mut commands: Commands| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetTagMessagesEvent { tag_id });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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
        .send(reqwest_request)
        .on_json_response(
            move |trigger: Trigger<JsonResponse<TagMessagesDtoResponse>>,
                  mut commands: Commands,
                  mut query_tags: Query<(Entity, &TagData, Option<&mut TagMessagesData>)>| {
                let dtos = match &trigger.0 {
                    Ok(dtos) => dtos,
                    Err(error_dto) => {
                        commands.trigger(ShowErrorEvent {
                            message: error_dto.message.clone(),
                        });
                        return;
                    }
                };

                let Some((entity, _, tag_messages)) = query_tags
                    .iter_mut()
                    .find(|(_, tag, _)| tag.dto.id == tag_id)
                else {
                    return;
                };

                match tag_messages {
                    Some(mut tag_messages) => {
                        tag_messages.dtos.clear();
                        tag_messages.dtos.extend(dtos.iter().cloned());
                    }
                    None => {
                        if let Some(mut entity) = commands.get_entity(entity) {
                            entity.insert(TagMessagesData { dtos: dtos.clone() });
                        }
                    }
                }
            },
        )
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
    let url = format!("{BACKEND_URL}/statuses/{:?}", trigger.project_id);

    let reqwest_request = client
        .get(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            |trigger: Trigger<JsonResponse<StatusesResponse>>,
             mut commands: Commands,
             query_status: Query<Entity, With<ProjectStatusesData>>| {
                match &trigger.0 {
                    Ok(dtos) => {
                        for entity in query_status.iter() {
                            commands.entity(entity).despawn_recursive();
                        }
                        commands.spawn(ProjectStatusesData { dtos: dtos.clone() });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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
    let project_id = trigger.dto.project_id;

    let reqwest_request = client
        .post(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            move |trigger: Trigger<JsonResponse<EmptyResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetStatusesEvent { project_id });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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
    let project_id = trigger.dto.project_id;

    let reqwest_request = client
        .put(url)
        .json(&trigger.dto)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            move |trigger: Trigger<JsonResponse<EmptyResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetStatusesEvent { project_id });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
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
    let project_id = trigger.project_id;

    let reqwest_request = client
        .delete(url)
        .header("authorization", get_token_from_user(query_user))
        .build()
        .unwrap();

    client
        .send(reqwest_request)
        .on_json_response(
            move |trigger: Trigger<JsonResponse<EmptyResponse>>, mut commands: Commands| {
                match &trigger.0 {
                    Ok(()) => {
                        commands.trigger(GetStatusesEvent { project_id });
                        commands.trigger(GetTagsEvent { project_id });
                    }
                    Err(error_dto) => commands.trigger(ShowErrorEvent {
                        message: error_dto.message.clone(),
                    }),
                };
            },
        )
        .on_error(on_reqwest_error);
}
