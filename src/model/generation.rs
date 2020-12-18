use super::*;
use noise::{OpenSimplex, Seedable};

impl Model {
    pub fn new(config: Config) -> Self {
        let (pack_list, resource_pack) = ResourcePack::load_resource_packs().unwrap();
        let rules = Rules {
            player_movement_speed: config.player_movement_speed,
            client_view_distance: config.view_distance,
            campfire_light: config.campfire_light,
            torch_light: config.torch_light,
            statue_light: config.statue_light,
            regeneration_percent: config.regeneration_percent,
            player_interaction_range: config.player_interaction_range,
            sound_distance: config.sound_distance,
            generation_distance: config.generation_distance,
        };
        let mut model = Self {
            pack_list,
            rules,
            generation_noises: Self::init_generation_noises(&resource_pack),
            resource_pack,
            ticks_per_second: config.ticks_per_second,
            chunk_size: config.chunk_size,
            chunks: HashMap::new(),
            players: HashMap::new(),
            items: HashMap::new(),
            current_time: 0,
            sounds: HashMap::new(),
        };
        model.generate_chunks_at(vec2(0, 0));
        model
    }
    pub fn new_player(&mut self) -> Id {
        let player_id;
        if let Some(pos) = self.get_spawnable_pos() {
            let player = Player {
                id: Id::new(),
                pos: pos.map(|x| x as f32),
                radius: 0.5,
                interaction_range: self.rules.player_interaction_range,
                item: None,
                colors: PlayerColors::new(),
                action: None,
            };
            player_id = player.id;
            self.sounds.insert(player.id, vec![]);
            self.players.insert(player.id, player);
        } else {
            error!("Did not find spawnable position");
            player_id = Id::new(); // TODO
        }
        player_id
    }
    pub fn spawn_item(&mut self, item_type: ItemType, pos: Vec2<f32>) {
        let item = Item {
            pos,
            size: self.resource_pack.items[&item_type].size,
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
    fn init_generation_noises(
        resource_pack: &ResourcePack,
    ) -> HashMap<GenerationParameter, GenerationNoise> {
        let mut noises = HashMap::new();
        for (biome_parameter, parameters) in &resource_pack.parameters {
            noises.insert(
                biome_parameter.clone(),
                GenerationNoise {
                    noise: Box::new(OpenSimplex::new().set_seed(global_rng().gen())),
                    parameters: parameters.clone(),
                },
            );
        }
        noises
    }
    pub fn generate_chunks_at(&mut self, origin_chunk_pos: Vec2<i64>) {
        self.generate_chunks_range(origin_chunk_pos, self.rules.generation_distance);
    }
    fn generate_chunks_range(&mut self, origin_chunk_pos: Vec2<i64>, generation_distance: usize) {
        let gen_dist = generation_distance as i64;
        for y in -gen_dist..gen_dist + 1 {
            for x in -gen_dist..gen_dist + 1 {
                let chunk_pos = vec2(x, y) + origin_chunk_pos;
                if !self.chunks.contains_key(&chunk_pos) {
                    self.generate_chunk(chunk_pos);
                }
            }
        }
    }
    fn generate_chunk(&mut self, chunk_pos: Vec2<i64>) {
        let mut tile_map = HashMap::new();
        for y in 0..self.chunk_size.y as i64 {
            for x in 0..self.chunk_size.x as i64 {
                let pos = Self::local_to_global_pos(self.chunk_size, chunk_pos, vec2(x, y));
                let biome = self.generate_biome(pos);
                let height = self.generation_noises[&GenerationParameter("Height".to_owned())]
                    .get(pos.map(|x| x as f32));
                tile_map.insert(vec2(x, y), Tile { pos, height, biome });
            }
        }
        self.chunks.insert(
            chunk_pos,
            Chunk {
                tile_map,
                items: HashMap::new(),
            },
        );
        for y in 0..self.chunk_size.y as i64 {
            for x in 0..self.chunk_size.x as i64 {
                let pos = Self::local_to_global_pos(self.chunk_size, chunk_pos, vec2(x, y));
                if self.is_empty_tile(pos) {
                    self.generate_tile(pos);
                }
            }
        }
    }
    fn generate_biome(&self, pos: Vec2<i64>) -> Biome {
        self.resource_pack
            .biomes
            .iter()
            .filter_map(|(biome, biome_gen)| {
                let mut total_score = 0.0;
                for (biome_parameter, noise) in &self.generation_noises {
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
            .unwrap()
            .0
            .clone()
    }
    fn generate_tile(&mut self, pos: Vec2<i64>) {
        let mut rng = global_rng();
        let choice = match self
            .resource_pack
            .item_generation
            .get(&self.get_tile(pos).unwrap().biome)
        {
            Some(gen) => gen
                .choose_weighted(&mut rng, |item| item.weight)
                .unwrap()
                .item_type
                .clone(),
            None => None,
        };
        if let Some(item_type) = choice {
            self.spawn_item(item_type, pos.map(|x| x as f32));
        }
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
    fn is_spawnable_tile(&self, pos: Vec2<i64>) -> bool {
        self.resource_pack.biomes[&self.get_tile(pos).unwrap().biome].spawnable
            && self.is_empty_tile(pos)
    }
    fn get_spawnable_pos(&self) -> Option<Vec2<f32>> {
        let mut positions = vec![];
        for (&chunk_pos, _) in &self.chunks {
            for y in 0..self.chunk_size.y as i64 {
                for x in 0..self.chunk_size.x as i64 {
                    let pos = Self::local_to_global_pos(self.chunk_size, chunk_pos, vec2(x, y));
                    if self.is_spawnable_tile(pos) {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Trans)]
pub struct BiomeGeneration {
    pub collidable: bool,
    pub spawnable: bool,
    pub parameters: HashMap<GenerationParameter, (f32, f32)>,
}

impl BiomeGeneration {
    pub fn get_distance(
        &self,
        pos: Vec2<f32>,
        parameter: &GenerationParameter,
        noise: &GenerationNoise,
    ) -> f32 {
        match self.parameters.get(parameter) {
            Some(parameter_zone) => {
                let noise_value = noise.get(pos);
                (noise_value - parameter_zone.0).min(parameter_zone.1 - noise_value)
            }
            None => noise.max_delta(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Trans)]
pub struct GenerationParameter(String);
