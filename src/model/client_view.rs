use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientView {
    pub players_online: usize,
    pub player_movement_speed: f32,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub players: Vec<Player>,
    pub items: Vec<Item>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
    pub item_properties: HashMap<ItemType, ItemProperties>,
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> ClientView {
        let player = self.players.get(&player_id).unwrap();

        let vision = ClientView {
            players_online: self.players.len(),
            player_movement_speed: self.rules.player_movement_speed,
            ticks_per_second: self.ticks_per_second,
            current_time: self.current_time,
            players: self
                .players
                .values()
                .filter(|other_player| {
                    player.id == other_player.id || player.load_area.contains(other_player.pos)
                })
                .cloned()
                .collect(),
            items: self
                .chunked_world
                .items()
                .filter(|item| player.load_area.contains(item.pos))
                .cloned()
                .collect(),
            recipes: self.resource_pack.recipes.clone(),
            sounds: mem::replace(self.sounds.get_mut(&player_id).unwrap(), vec![]),
            item_properties: self.resource_pack.item_properties.clone(),
        };
        vision
    }
}
