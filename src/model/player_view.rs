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
        for light_source in self.items.values().filter(|item| {
            item.item_type == ItemType::Campfire
                || item.item_type == ItemType::Statue
                || item.item_type == ItemType::Torch
        }) {
            Self::add_view_radius(
                &mut view,
                light_source.pos.map(|x| x as f32),
                match light_source.item_type {
                    ItemType::Campfire => self.rules.campfire_light,
                    ItemType::Statue => self.rules.statue_light,
                    ItemType::Torch => self.rules.torch_light,
                    _ => unreachable!(),
                },
            );
        }
        for entity_torch in self.entities.values().filter(|entity| match entity.item {
            Some(item) => item == ItemType::Torch,
            _ => false,
        }) {
            Self::add_view_radius(&mut view, entity_torch.pos, self.rules.torch_light);
        }

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
            recipes: self.recipes.clone(),
            sounds: mem::replace(self.sounds.get_mut(&player_id).unwrap(), vec![]),
        };
        vision
    }
}
