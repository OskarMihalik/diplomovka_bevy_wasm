use std::process::Command;

use bevy::prelude::*;
use bevy_mod_reqwest::*;
use dto::default::{DtoResponse, NewTagDto, TagDto, TagDtoResponse, Test};

use crate::{
    building::{RebuildTagsEvent, TagData},
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
            .add_observer(create_new_tag);
    }
}

#[derive(Event)]
pub struct GetTagsEvent {}

// fn get_tags(_trigger: Trigger<GetTagsEvent>, mut client: BevyReqwest) {
//     let url = format!("{BACKEND_URL}/tags");

//     // use regular reqwest http calls, then poll them to completion.
//     let reqwest_request = client.get(url).build().unwrap();

//     client
//         // Sends the created http request
//         .send(reqwest_request)
//         // The response from the http request can be reached using an observersystem,
//         // where the only requirement is that the first parameter in the system is the specific Trigger type
//         // the rest is the same as a regular system
//         .on_response(
//             |trigger: Trigger<ReqwestResponseEvent>,
//              mut commands: Commands |{
//                 let response = trigger.event();
//                 let status = response.status();
//                 let data = response.as_str().unwrap();
//                 bevy::log::info!("response: {status}, data: {data}");
//                 let parsed: TagDtoResponse = serde_json::from_str(data).unwrap();
//                 match parsed {
//                     DtoResponse::Ok(tags) => {
//                         commands.trigger(RebuildTagsEvent {
//                             new_tag_dtos: tags,
//                             parent_entity: ,
//                         });
//                     }
//                     DtoResponse::Err(error_dto) => commands.trigger(ShowErrorEvent {
//                         message: error_dto.message,
//                     }),
//                 };
//             },
//         )
//         // In case of request error, it can be reached using an observersystem as well
//         .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
//             let e = &trigger.event().0;
//             bevy::log::info!("error: {e:?}");
//         });
// }

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
            move |trigger: Trigger<ReqwestResponseEvent>,
                  mut ui_state: ResMut<UiState>,
                  mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>,
                  query_tags: Query<(Entity, &TagData)>| {
                let response = trigger.event();
                let status = response.status();
                let data = response.as_str().unwrap();
                bevy::log::info!("response: {status}, data: {data}");
                let parsed: TagDtoResponse = serde_json::from_str(data).unwrap();
                match parsed {
                    DtoResponse::Ok(tags) => {
                        for old_tag in query_tags.iter() {
                            commands.entity(old_tag.0).despawn_recursive();
                        }
                        bevy::log::info!("tags: {:#?}", tags);
                        for tag_dto in tags.iter() {
                            commands
                                .spawn((
                                    Mesh3d(meshes.add(Cuboid::new(0.3, 0.3, 0.3))),
                                    MeshMaterial3d(materials.add(StandardMaterial {
                                        base_color: Color::srgb(1.0, 0.0, 0.0),
                                        ..Default::default()
                                    })),
                                    GlobalTransform::from_xyz(
                                        tag_dto.position_x,
                                        tag_dto.position_y,
                                        tag_dto.position_z,
                                    ),
                                    TagData {
                                        dto: TagDto {
                                            id: tag_dto.id,
                                            title: tag_dto.title.clone(),
                                            model_id: tag_dto.model_id,
                                            created_at: tag_dto.created_at.clone(),
                                            position_x: tag_dto.position_x,
                                            position_y: tag_dto.position_y,
                                            position_z: tag_dto.position_z,
                                        },
                                    },
                                ))
                                .set_parent_in_place(parent_entity);
                        }
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
