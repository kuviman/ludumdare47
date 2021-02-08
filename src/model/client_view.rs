use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientView {
    pub players_online: usize,
    pub player_movement_speed: f32,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub entities: Vec<Entity>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
    pub item_properties: HashMap<EntityType, EntityProperties>,
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> ClientView {
        let entity = self.chunked_world.get_entity(player_id).unwrap();
        let player = entity.components.player.as_ref().unwrap();

        let vision = ClientView {
            players_online: self
                .chunked_world
                .entities()
                .filter(|e| e.components.player.is_some())
                .count(),
            player_movement_speed: self.rules.player_movement_speed,
            ticks_per_second: self.ticks_per_second,
            current_time: self.current_time,
            entities: self
                .chunked_world
                .entities()
                .filter(|e| e.id == entity.id || player.load_area.contains(e.pos))
                .cloned()
                .collect(),
            recipes: self.resource_pack.recipes.clone(),
            sounds: mem::replace(self.sounds.get_mut(&player_id).unwrap(), vec![]),
            item_properties: self.resource_pack.entity_properties.clone(),
        };
        vision
    }
}
