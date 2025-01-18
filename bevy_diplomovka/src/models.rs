use std::collections::HashMap;

use bevy::prelude::*;
use dto::model::ModelDto;

use crate::building::ModelData;

pub struct ModelManagmentPlugin;

impl Plugin for ModelManagmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(update_models);
    }
}

#[derive(Event)]
pub struct UpdateModelsEvent {
    pub dtos: Vec<ModelDto>,
}

fn update_models(
    trigger: Trigger<UpdateModelsEvent>,
    mut commands: Commands,
    query_models: Query<(Entity, &ModelData)>,
) {
    let mut dto_map: HashMap<i32, ModelDto> = HashMap::new();
    for dto in trigger.dtos.iter() {
        dto_map.insert(dto.id, dto.clone());
    }

    for (entity, model_data) in query_models.iter() {
        let new_data = match dto_map.get(&model_data.dto.id) {
            Some(ok) => ok.clone(),
            None => {
                commands.entity(entity).despawn_recursive();
                dto_map.remove(&model_data.dto.id);
                continue;
            }
        };
        dto_map.remove(&new_data.id);
        commands
            .entity(entity)
            .remove::<ModelData>()
            .insert(ModelData { dto: new_data });
    }
    for (_key, dto) in dto_map {
        bevy::log::info!("spawning model: {:?}", dto);
        commands.spawn(ModelData { dto });
    }
}
