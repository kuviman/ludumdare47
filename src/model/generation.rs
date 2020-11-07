use super::*;

impl Model {
    pub fn new(config: Config) -> Self {
        let recipes = Config::default_recipes();
        let (tiles, height_map) = Self::generate_map(config.map_size, Config::default_biomes());
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
            size: config.map_size,
            tiles,
            height_map,
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
        for y in 0..model.size.y {
            for x in 0..model.size.x {
                let pos = vec2(x as i64, y as i64);
                if model.is_empty_tile(pos) {
                    model.generate_tile(pos);
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
        if let Some(pos) = Some(vec2(20.0, 20.0)) {
            //self.get_spawnable_pos(Biome::Forest) {
            let entity = Entity {
                id: Id::new(),
                pos: pos.map(|x| x as f32),
                radius: 0.5,
                view_range: self.calc_view_range(),
                interaction_range: self.rules.entity_interaction_range,
                item: None,
                colors: EntityColors::new(),
                move_to: None,
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
    fn generate_map(
        map_size: Vec2<usize>,
        biomes: HashMap<Biome, BiomeGeneration>,
    ) -> (HashMap<Vec2<i64>, Tile>, HashMap<Vec2<i64>, f32>) {
        let mut noise_storage = HashMap::new();
        let mut noises = HashMap::new();
        for (_, biome_generation) in &biomes {
            for &setting in biome_generation
                .parameters
                .keys()
                .filter(|&setting| !noises.contains_key(setting))
            {
                noise_storage.insert(setting, OpenSimplex::new().set_seed(global_rng().gen()));
            }
        }
        for (&setting, noise) in &noise_storage {
            noises.insert(setting, noise as &dyn NoiseFn<[f64; 2]>);
        }

        let mut tiles_height_map = HashMap::new();
        let mut tiles = HashMap::new();
        for y in 0..map_size.y as i64 {
            for x in 0..map_size.x as i64 {
                let pos = vec2(x, y);
                let height = noises[&BiomeParameters::Height]
                    .get([pos.x as f64 / 20.0, pos.y as f64 / 20.0])
                    as f32;
                tiles_height_map.insert(pos, height);
                tiles.insert(
                    pos,
                    Tile {
                        pos,
                        height,
                        biome: Self::generate_biome(pos, None, &noises, &biomes).unwrap(),
                    },
                );
            }
        }
        let mut height_map =
            HashMap::with_capacity(tiles_height_map.len() + map_size.x + map_size.y + 1);
        for y in 1..map_size.y as i64 {
            for x in 1..map_size.x as i64 {
                let height = (tiles_height_map[&vec2(x, y)]
                    + tiles_height_map[&vec2(x - 1, y)]
                    + tiles_height_map[&vec2(x, y - 1)]
                    + tiles_height_map[&vec2(x - 1, y - 1)])
                    / 4.0;
                height_map.insert(vec2(x, y), height);
            }
        }
        for y in 1..map_size.y as i64 {
            // Right
            let height = (tiles_height_map[&vec2(map_size.x as i64 - 1, y)]
                + tiles_height_map[&vec2(map_size.x as i64 - 1, y - 1)])
                / 2.0;
            height_map.insert(vec2(map_size.x as i64, y), height);
            // Left
            let height = (tiles_height_map[&vec2(0, y)] + tiles_height_map[&vec2(0, y - 1)]) / 2.0;
            height_map.insert(vec2(0, y), height);
        }
        for x in 1..map_size.x as i64 {
            // Top
            let height = (tiles_height_map[&vec2(x, map_size.y as i64 - 1)]
                + tiles_height_map[&vec2(x - 1, map_size.y as i64 - 1)])
                / 2.0;
            height_map.insert(vec2(x, map_size.y as i64), height);
            // Bottom
            let height = (tiles_height_map[&vec2(x, 0)] + tiles_height_map[&vec2(x - 1, 0)]) / 2.0;
            height_map.insert(vec2(x, 0), height);
        }
        // Top-right
        height_map.insert(
            vec2(map_size.x as i64, map_size.y as i64),
            tiles_height_map[&vec2(map_size.x as i64 - 1, map_size.y as i64 - 1)],
        );
        // Top-left
        height_map.insert(
            vec2(0, map_size.y as i64),
            tiles_height_map[&vec2(0, map_size.y as i64 - 1)],
        );
        // Bottom-right
        height_map.insert(
            vec2(map_size.x as i64, 0),
            tiles_height_map[&vec2(map_size.x as i64 - 1, 0)],
        );
        // Bottom-left
        height_map.insert(vec2(0, 0), tiles_height_map[&vec2(0, 0)]);
        (tiles, height_map)
    }
    fn generate_biome(
        pos: Vec2<i64>,
        parent_biome: Option<Biome>,
        noises: &HashMap<BiomeParameters, &dyn NoiseFn<[f64; 2]>>,
        biomes: &HashMap<Biome, BiomeGeneration>,
    ) -> Option<Biome> {
        match biomes
            .iter()
            .filter(|(&biome, biome_generation)| {
                biome_generation.parent_biome == parent_biome || Some(biome) == parent_biome
            })
            .map(|(&biome, biome_generation)| {
                (biome, biome_generation.calculate_score(biome, pos, noises))
            })
            .max_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap())
        {
            Some((biome, _)) => {
                if Some(biome) == parent_biome {
                    parent_biome
                } else {
                    Self::generate_biome(pos, Some(biome), noises, biomes)
                }
            }
            None => parent_biome,
        }
    }
    pub fn generate_tile(&mut self, pos: Vec2<i64>) {
        let mut rng = global_rng();
        let choice = self.generation_choices[&self.tiles.get(&pos).unwrap().biome]
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0;
        if let Some(item_type) = choice {
            self.spawn_item(item_type, pos.map(|x| x as f32));
        }
    }
    fn is_spawnable_tile(&self, pos: Vec2<i64>) -> bool {
        self.tiles.get(&pos).unwrap().biome != Biome::Lake && self.is_empty_tile(pos)
    }
    fn get_spawnable_pos(&self, ground_type: Biome) -> Option<Vec2<f32>> {
        let mut positions = vec![];
        for y in 0..self.size.y as i64 {
            for x in 0..self.size.x as i64 {
                let pos = vec2(x, y);
                if self.is_spawnable_tile(pos) && self.tiles.get(&pos).unwrap().biome == ground_type
                {
                    positions.push(vec2(x as f32, y as f32));
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
}
