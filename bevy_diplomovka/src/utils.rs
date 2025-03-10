use bevy::prelude::*;
use time::PrimitiveDateTime;

use crate::users::LoggedUser;

#[derive(Event)]
pub struct LogEntityComponents {
    pub entity: Entity,
}

pub fn log_entity_components(trigger: Trigger<LogEntityComponents>, world: &World) {
    let target_entity = trigger.entity;
    let components: Vec<_> = world.inspect_entity(target_entity).cloned().collect();
    bevy::log::info!("{:#?}", components);
}

pub fn compare_by_created_at(
    value_1: &PrimitiveDateTime,
    value_2: &PrimitiveDateTime,
) -> std::cmp::Ordering {
    if value_1 > value_2 {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}

pub fn get_token_from_user(query_user: Option<Single<(Entity, &LoggedUser)>>) -> String {
    let token = match query_user {
        Some(user) => user.1.dto.token.clone(),
        None => "".to_string(),
    };
    return token;
}

pub fn convert_color_to_egui(color: f32) -> u8 {
    (color * 255.0) as u8
}
