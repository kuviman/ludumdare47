use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Chunk {
    pub tile_map: HashMap<Vec2<i64>, Tile>,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let recipes = Config::default_recipes();
        let chunks = Self::generate_map(&config);
        let rules = Rules {
            entity_movement_speed: config.player_movement_speed,
            entity_day_view_distance: config.player_day_view_distance,
            entity_night_view_distance: config.player_night_view_distance,
            campfire_light: config.campfire_light,
            torch_light: config.torch_light,
            statue_light: config.statue_light,
            fire_extinguish_chance: config.fire_extinguish_chance,
            regeneration_percent: config.regeneration_percent,
            entity_interaction_range: config.entity_interaction_range,
        };
        let mut model = Self {
            rules,
            score: 0,
            ticks_per_second: config.ticks_per_second,
            chunk_size: config.chunk_size,
            chunks,
            entities: HashMap::new(),
            items: HashMap::new(),
            current_time: 0,
            day_length: config.day_length,
            night_length: config.night_length,
            recipes,
            scores_map: Config::default_scores_map(),
            sound_distance: config.sound_distance,
            generation_choices: Config::default_generation_choices(),
            sounds: HashMap::new(),
        };
        for chunk_pos in model.chunks.keys().copied().collect::<Vec<Vec2<i64>>>() {
            for y in 0..model.chunk_size.y as i64 {
                for x in 0..model.chunk_size.x as i64 {
                    let pos = Self::local_to_global_pos(model.chunk_size, chunk_pos, vec2(x, y));
                    if model.is_empty_tile(pos) {
                        model.generate_tile(pos);
                    }
                }
            }
        }
        if let Some(pos) = model.get_spawnable_pos(Biome::Forest) {
            model.spawn_item(ItemType::Statue, pos);
        } else {
            error!("Did not find a position for a statue");
        }
        model
    }
    pub fn new_player(&mut self) -> Id {
        let player_id;
        if let Some(pos) = self.get_spawnable_pos(Biome::Forest) {
            let entity = Entity {
                id: Id::new(),
                pos: pos.map(|x| x as f32),
                radius: 0.5,
                view_range: self.calc_view_range(),
                interaction_range: self.rules.entity_interaction_range,
                item: None,
                colors: EntityColors::new(),
                action: None,
            };
            player_id = entity.id;
            self.sounds.insert(entity.id, vec![]);
            self.entities.insert(entity.id, entity);
        } else {
            error!("Did not find spawnable position");
            player_id = Id::new(); // TODO
        }
        player_id
    }
    pub fn spawn_item(&mut self, item_type: ItemType, pos: Vec2<f32>) {
        let item = Item {
            pos,
            size: item_type.size(),
            item_type,
        };
        self.items.insert(Id::new(), item);
    }
    pub fn remove_item(&mut self, pos: Vec2<f32>, range: f32) -> Option<Item> {
        match self
            .items
            .iter_mut()
            .find(|(_, item)| (item.pos - pos).len() <= range)
        {
            Some((index, _)) => {
                let index = index.clone();
                self.items.remove(&index)
            }
            None => None,
        }
    }
    fn generate_map(config: &Config) -> HashMap<Vec2<i64>, Chunk> {
        let noise = OpenSimplex::new().set_seed(global_rng().gen());
        let noise2 = OpenSimplex::new().set_seed(global_rng().gen());

        let mut chunks = HashMap::new();
        for y in 0..config.initial_generation_size.y as i64 {
            for x in 0..config.initial_generation_size.x as i64 {
                chunks.insert(
                    vec2(x, y),
                    Self::generate_chunk(config, vec2(x, y), &noise, &noise2),
                );
            }
        }
        chunks
    }
    fn generate_chunk(
        config: &Config,
        chunk_pos: Vec2<i64>,
        noise: &dyn NoiseFn<[f64; 2]>,
        noise2: &dyn NoiseFn<[f64; 2]>,
    ) -> Chunk {
        let mut tile_map = HashMap::new();
        for y in 0..config.chunk_size.y as i64 {
            for x in 0..config.chunk_size.x as i64 {
                let pos = Self::local_to_global_pos(config.chunk_size, chunk_pos, vec2(x, y))
                    .map(|x| x as f32);
                // let normalized_pos = vec2(pos.x / 250.0, pos.y / 250.0) * 2.0 - vec2(1.0, 1.0);
                // let height = 1.0 - normalized_pos.len() * 1.2
                //     + (noise.get([normalized_pos.x as f64 * 5.0, normalized_pos.y as f64 * 5.0])
                //         as f32
                //         / 10.0);
                let height = 0.3; // height.min(0.3);
                tile_map.insert(
                    vec2(x, y),
                    Tile {
                        height,
                        biome: if height < 0.0 {
                            Biome::Water
                        } else if height < 0.05 {
                            Biome::Beach
                        } else {
                            if noise2.get([pos.x as f64 / 10.0, pos.y as f64 / 10.0]) > 0.2 {
                                Biome::Hills
                            } else if noise.get([pos.x as f64 / 10.0, pos.y as f64 / 10.0]) > 0.2
                                && noise2.get([pos.x as f64 / 20.0 + 100.0, pos.y as f64 / 20.0])
                                    > 0.2
                            {
                                Biome::MagicForest
                            } else {
                                Biome::Forest
                            }
                        },
                    },
                );
            }
        }
        Chunk { tile_map }
    }
    pub fn get_tile(&self, pos: Vec2<i64>) -> Option<&Tile> {
        match self.get_chunk(pos) {
            Some(chunk) => {
                let tile_pos = vec2(
                    pos.x % self.chunk_size.x as i64,
                    pos.y % self.chunk_size.y as i64,
                );
                Some(&chunk.tile_map[&tile_pos])
            }
            None => None,
        }
    }
    pub fn get_chunk(&self, pos: Vec2<i64>) -> Option<&Chunk> {
        let mut chunk_pos = vec2(
            pos.x / self.chunk_size.x as i64,
            pos.y / self.chunk_size.y as i64,
        );
        if pos.x < 0 {
            chunk_pos.x -= 1;
        }
        if pos.y < 0 {
            chunk_pos.y -= 1;
        }
        self.chunks.get(&chunk_pos)
    }
    pub fn generate_tile(&mut self, pos: Vec2<i64>) {
        let mut rng = global_rng();
        let choice = self.generation_choices[&self.get_tile(pos).unwrap().biome]
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0;
        if let Some(item_type) = choice {
            self.spawn_item(item_type, pos.map(|x| x as f32));
        }
    }
    fn is_spawnable_tile(&self, pos: Vec2<i64>) -> bool {
        self.get_tile(pos).unwrap().biome != Biome::Water && self.is_empty_tile(pos)
    }
    fn get_spawnable_pos(&self, ground_type: Biome) -> Option<Vec2<f32>> {
        let mut positions = vec![];
        for (&chunk_pos, _) in &self.chunks {
            for y in 0..self.chunk_size.y as i64 {
                for x in 0..self.chunk_size.x as i64 {
                    let pos = Self::local_to_global_pos(self.chunk_size, chunk_pos, vec2(x, y));
                    if self.is_spawnable_tile(pos)
                        && self.get_tile(pos).unwrap().biome == ground_type
                    {
                        positions.push(pos.map(|x| x as f32));
                    }
                }
            }
        }
        let length = positions.len();
        if length > 0 {
            positions.get(global_rng().gen_range(0, length)).copied()
        } else {
            None
        }
    }
    pub fn local_to_global_pos(
        chunk_size: Vec2<usize>,
        chunk_pos: Vec2<i64>,
        pos: Vec2<i64>,
    ) -> Vec2<i64> {
        vec2(
            chunk_pos.x * chunk_size.x as i64 + pos.x,
            chunk_pos.y * chunk_size.y as i64 + pos.y,
        )
    }
}
