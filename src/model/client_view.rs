use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientView {
    pub players_online: usize,
    pub player_movement_speed: f32,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub tiles: HashMap<Vec2<i64>, Tile>,
    pub players: Vec<Player>,
    pub items: Vec<Item>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
    pub item_parameters: HashMap<ItemType, ItemParameters>,
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> ClientView {
        let player = self.players.get(&player_id).unwrap();

        let vision = ClientView {
            players_online: self.players.len(),
            player_movement_speed: self.rules.player_movement_speed,
            ticks_per_second: self.ticks_per_second,
            current_time: self.current_time,
            tiles: {
                let mut tiles = HashMap::new();
                for pos in player.load_area.map(|x| x as i64).points() {
                    if let Some(tile) = self.chunked_world.get_tile(pos) {
                        tiles.insert(pos, tile.clone());
                    }
                }
                tiles
            },
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
            item_parameters: self.resource_pack.items.clone(),
        };
        vision
    }
}
