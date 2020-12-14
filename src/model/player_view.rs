use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct PlayerView {
    pub players_online: usize,
    pub entity_movement_speed: f32,
    pub score: i32,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub tiles: HashMap<Vec2<i64>, Tile>,
    pub entities: Vec<Entity>,
    pub items: HashMap<Id, Item>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> PlayerView {
        let entity = self.entities.get(&player_id).unwrap();
        let mut view = HashSet::new();
        Self::add_view_radius(&mut view, entity.pos, self.rules.entity_view_distance);

        let vision = PlayerView {
            players_online: self.entities.len(),
            entity_movement_speed: self.rules.entity_movement_speed,
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
            entities: self
                .entities
                .iter()
                .filter(|(_, entity)| view.contains(&entity.pos.map(|x| x as i64)))
                .map(|(_, entity)| entity.clone())
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
