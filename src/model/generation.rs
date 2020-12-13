use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Chunk {
    pub tile_map: HashMap<Vec2<i64>, Tile>,
    pub items: HashMap<Id, Item>,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let resource_pack = Config::load_resource_packs().unwrap();
        let recipes = Config::default_recipes();
        let chunks = Self::generate_map(&config, &resource_pack);
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
        let id = Id::new();
        self.items.insert(id, item.clone());
        self.chunks
            .get_mut(&self.get_chunk_pos(pos.map(|x| x as i64)))
            .unwrap()
            .items
            .insert(id, item);
    }
    pub fn remove_item_id(&mut self, id: Id) -> Option<Item> {
        let item = self.items.remove(&id);
        if let Some(item) = item {
            self.chunks
                .get_mut(&self.get_chunk_pos(item.pos.map(|x| x as i64)))
                .unwrap()
                .items
                .remove(&id);
            Some(item)
        } else {
            None
        }
    }
    pub fn remove_item(&mut self, pos: Vec2<f32>, range: f32) -> Option<Item> {
        match self
            .items
            .iter_mut()
            .find(|(_, item)| (item.pos - pos).len() <= range)
        {
            Some((index, _)) => {
                let index = index.clone();
                self.chunks
                    .get_mut(&self.get_chunk_pos(pos.map(|x| x as i64)))
                    .unwrap()
                    .items
                    .remove(&index);
                self.items.remove(&index)
            }
            None => None,
        }
    }
    fn generate_map(config: &Config, resource_pack: &ResourcePack) -> HashMap<Vec2<i64>, Chunk> {
        let mut noises = HashMap::new();
        for (&biome_parameter, parameters) in &resource_pack.parameters {
            noises.insert(
                biome_parameter,
                Noise {
                    noise: Box::new(OpenSimplex::new().set_seed(global_rng().gen())),
                    parameters: parameters.clone(),
                },
            );
        }

        let mut chunks = HashMap::new();
        let gen_y = config.initial_generation_size.y as i64 / 2;
        let gen_x = config.initial_generation_size.x as i64 / 2;
        for y in -gen_y..gen_y + 1 {
            for x in -gen_x..gen_x + 1 {
                chunks.insert(
                    vec2(x, y),
                    Self::generate_chunk(config, vec2(x, y), &noises, &resource_pack.biomes),
                );
            }
        }
        chunks
    }
    fn generate_chunk(
        config: &Config,
        chunk_pos: Vec2<i64>,
        noises: &HashMap<BiomeParameter, Noise>,
        biomes: &HashMap<Biome, BiomeGeneration>,
    ) -> Chunk {
        let mut tile_map = HashMap::new();
        for y in 0..config.chunk_size.y as i64 {
            for x in 0..config.chunk_size.x as i64 {
                let pos = Self::local_to_global_pos(config.chunk_size, chunk_pos, vec2(x, y));
                let biome = Self::generate_biome(pos, noises, biomes);
                let height = biome.height();
                tile_map.insert(vec2(x, y), Tile { pos, height, biome });
            }
        }
        Chunk {
            tile_map,
            items: HashMap::new(),
        }
    }
    fn generate_biome(
        pos: Vec2<i64>,
        noises: &HashMap<BiomeParameter, Noise>,
        biomes: &HashMap<Biome, BiomeGeneration>,
    ) -> Biome {
        *biomes
            .iter()
            .filter_map(|(biome, biome_gen)| {
                let mut total_score = 0.0;
                for (biome_parameter, noise) in noises {
                    let score =
                        biome_gen.get_distance(pos.map(|x| x as f32), biome_parameter, noise);
                    if score < 0.0 {
                        return None;
                    } else {
                        total_score += score;
                    }
                }
                Some((biome, total_score))
            })
            .min_by_key(|(_, score)| r32(*score))
            .unwrap_or((&Biome::Void, 0.0))
            .0
    }
    pub fn get_tile(&self, pos: Vec2<i64>) -> Option<&Tile> {
        let chunk_pos = self.get_chunk_pos(pos.map(|x| x as i64));
        match self.chunks.get(&chunk_pos) {
            Some(chunk) => {
                let tile_pos = vec2(
                    pos.x - chunk_pos.x * self.chunk_size.x as i64,
                    pos.y - chunk_pos.y * self.chunk_size.y as i64,
                );
                Some(&chunk.tile_map[&tile_pos])
            }
            None => None,
        }
    }
    pub fn get_chunk_pos(&self, pos: Vec2<i64>) -> Vec2<i64> {
        let x = if pos.x >= 0 {
            pos.x / self.chunk_size.x as i64
        } else {
            (pos.x + 1) / self.chunk_size.x as i64 - 1
        };
        let y = if pos.y >= 0 {
            pos.y / self.chunk_size.y as i64
        } else {
            (pos.y + 1) / self.chunk_size.y as i64 - 1
        };
        vec2(x, y)
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
        self.get_tile(pos).unwrap().biome != Biome::Lake && self.is_empty_tile(pos)
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
