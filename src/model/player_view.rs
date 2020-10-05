use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct PlayerView {
    pub players_online: usize,
    pub score: i32,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub day_length: usize,
    pub night_length: usize,
    pub height_map: Vec<Vec<f32>>,
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub structures: Vec<Structure>,
    pub recipes: Vec<Recipe>,
    pub sounds: Vec<Sound>,
}

impl Model {
    pub fn get_view(&mut self, player_id: Id) -> PlayerView {
        let mut timer = Timer::new();
        let entity = self.entities.get(&player_id).unwrap();
        let mut view = HashSet::new();
        Self::add_view_radius(&mut view, entity.pos, entity.view_range);
        for light_source in self.structures.values().filter(|structure| {
            structure.structure_type == StructureType::Campfire
                || structure.structure_type == StructureType::Statue
                || structure.structure_type == StructureType::Item { item: Item::Torch }
        }) {
            Self::add_view_radius(
                &mut view,
                light_source.pos,
                match light_source.structure_type {
                    StructureType::Campfire => self.rules.campfire_light,
                    StructureType::Statue => self.rules.statue_light,
                    StructureType::Item { item: Item::Torch } => self.rules.torch_light,
                    _ => unreachable!(),
                },
            );
        }
        for entity_torch in self.entities.values().filter(|entity| match entity.item {
            Some(item) => item == Item::Torch,
            _ => false,
        }) {
            Self::add_view_radius(&mut view, entity_torch.pos, self.rules.torch_light);
        }

        let vision = PlayerView {
            players_online: self.entities.len(),
            score: self.score,
            ticks_per_second: self.ticks_per_second,
            current_time: self.current_time,
            day_length: self.day_length,
            night_length: self.night_length,
            tiles: {
                let mut tiles = vec![];
                for y in 0..self.size.y {
                    let tile_row = self.tiles.get(y).unwrap();
                    for x in 0..self.size.x {
                        let pos = vec2(x, y);
                        if view.contains(&pos) {
                            tiles.push(tile_row.get(x).unwrap().clone());
                        }
                    }
                }
                tiles
            },
            height_map: self.height_map.clone(),
            entities: self
                .entities
                .iter()
                .filter(|(_, entity)| view.contains(&entity.pos))
                .map(|(_, entity)| entity.clone())
                .collect(),
            structures: self
                .structures
                .values()
                .filter(|structure| view.contains(&structure.pos))
                .map(|structure| structure.clone())
                .collect(),
            recipes: self.recipes.clone(),
            sounds: mem::replace(self.sounds.get_mut(&player_id).unwrap(), vec![]),
        };
        println!("Got player view in {:?}", timer.tick());
        vision
    }
}
