use bevy::prelude::*;
use time::PrimitiveDateTime;

use crate::users::LoggedUser;

#[derive(Event)]
pub struct LogEntityComponents {
    pub entity: Entity,
}

pub fn log_entity_components(trigger: On<LogEntityComponents>, world: &World) {
    let target_entity = trigger.entity;
    let Ok(components) = world.inspect_entity(target_entity) else {
        return;
    };
    let components: Vec<_> = components.cloned().collect();
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

pub fn filter_tags(title: &str, status: &str, filter_title: &str, filter_status: &str) -> bool {
    if filter_title.is_empty() && filter_status.is_empty() {
        return true;
    }
    if filter_title.is_empty() {
        return status == filter_status;
    }
    if filter_status.is_empty() {
        return title.contains(filter_title);
    }
    return title.contains(filter_title) && status == filter_status;
}
