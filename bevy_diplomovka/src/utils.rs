use bevy::prelude::*;

#[derive(Event)]
pub struct LogEntityComponents {
    pub entity: Entity,
}

pub fn log_entity_components(trigger: Trigger<LogEntityComponents>, world: &World) {
    let target_entity = trigger.entity;
    let components: Vec<_> = world.inspect_entity(target_entity).cloned().collect();
    bevy::log::info!("{:#?}", components);
}
