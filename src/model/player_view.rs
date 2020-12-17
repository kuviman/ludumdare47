use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct PlayerView {
    pub players_online: usize,
    pub player_movement_speed: f32,
    pub score: i32,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub tiles: HashMap<Vec2<i64>, Tile>,
    pub players: Vec<Player>,
    pub items: HashMap<Id, Item>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> PlayerView {
        let player = self.players.get(&player_id).unwrap();
        let mut view = HashSet::new();
        Self::add_view_radius(&mut view, player.pos, self.rules.player_view_distance);

        let vision = PlayerView {
            players_online: self.players.len(),
            player_movement_speed: self.rules.player_movement_speed,
            score: self.score,
            ticks_per_second: self.ticks_per_second,
            current_time: self.current_time,
            tiles: {
                let mut tiles = HashMap::new();
                for &pos in &view {
                    if let Some(tile) = self.get_tile(pos) {
                        tiles.insert(pos, tile.clone());
                    }
                }
                tiles
            },
            players: self
                .players
                .iter()
                .filter(|(_, player)| view.contains(&player.pos.map(|x| x as i64)))
                .map(|(_, player)| player.clone())
                .collect(),
            items: self
                .items
                .iter()
                .filter(|(_, item)| view.contains(&item.pos.map(|x| x as i64)))
                .map(|(id, item)| (id.clone(), item.clone()))
                .collect(),
            recipes: self.resource_pack.recipes.clone(),
            sounds: mem::replace(self.sounds.get_mut(&player_id).unwrap(), vec![]),
        };
        vision
    }
}
