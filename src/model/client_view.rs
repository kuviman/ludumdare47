use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientView {
    pub players_online: usize,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub entities: Vec<Entity>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
}

impl ClientView {
    pub fn get_closest_entity(&self, pos: Vec2<f32>) -> Option<&Entity> {
        self.entities
            .iter()
            .filter(|entity| entity.pos.is_some() && entity.size.is_some())
            .find(|entity| (entity.pos.unwrap() - pos).len() <= entity.size.unwrap())
    }
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> ClientView {
        let entity = self.chunked_world.get_entity(player_id).unwrap();

        let vision = ClientView {
            players_online: self
                .chunked_world
                .entities()
                .filter(|e| match &e.components.controller {
                    Some(CompController::Player { .. }) => true,
                    _ => false,
                })
                .count(),
            ticks_per_second: self.ticks_per_second,
            current_time: self.current_time,
            entities: self
                .chunked_world
                .entities()
                .filter(|e| {
                    e.id == entity.id
                        || e.pos.is_some()
                            && entity
                                .load_area
                                .as_ref()
                                .unwrap()
                                .load_area
                                .contains(e.pos.unwrap())
                })
                .cloned()
                .collect(),
            recipes: self.resource_pack.recipes.clone(),
            sounds: mem::replace(self.sounds.get_mut(&player_id).unwrap(), vec![]),
        };
        vision
    }
}
