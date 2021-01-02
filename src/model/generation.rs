use super::*;
use noise::{NoiseFn, OpenSimplex, Seedable};

impl Model {
    pub fn new_player(&mut self) -> Id {
        let player_id;
        if let Some(pos) = self.get_spawnable_pos(vec2(0, 0), self.rules.spawn_area) {
            let player = Player {
                id: Id::new(),
                pos: pos.map(|x| x as f32),
                radius: 0.5,
                interaction_range: self.rules.player_interaction_range,
                item: None,
                colors: PlayerColors::new(),
                action: None,
                load_area: AABB::pos_size(pos.map(|x| x as f32), vec2(0.0, 0.0)),
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
        self.loaded_chunks
            .get_mut(&self.get_chunk_pos(pos.map(|x| x as i64)))
            .unwrap()
            .items
            .insert(id, item);
    }
    pub fn remove_item_id(&mut self, id: Id) -> Option<Item> {
        let item = self.items.remove(&id);
        if let Some(item) = item {
            self.loaded_chunks
                .get_mut(&self.get_chunk_pos(item.pos.map(|x| x as i64)))
                .unwrap()
                .items
                .remove(&id);
            Some(item)
        } else {
            None
        }
    }
    pub fn init_generation_noises(
        seed: u32,
        resource_pack: &ResourcePack,
    ) -> HashMap<GenerationParameter, GenerationNoise> {
        let seed_noise = OpenSimplex::new().set_seed(seed);
        let mut noises = HashMap::new();
        for (biome_parameter, parameters) in &resource_pack.parameters {
            noises.insert(
                biome_parameter.clone(),
                GenerationNoise {
                    noise: Box::new(OpenSimplex::new().set_seed(
                        (seed_noise.get([hash(biome_parameter) as f64, 0.0]) * 1000.0) as u32,
                    )),
                    parameters: parameters.clone(),
                },
            );
            fn hash<T>(obj: T) -> u64
            where
                T: std::hash::Hash,
            {
                use std::hash::*;
                let mut hasher = siphasher::sip::SipHasher::new();
                obj.hash(&mut hasher);
                hasher.finish()
            }
        }
        noises
    }
    pub fn load_chunks_at(&mut self, origin_chunk_pos: Vec2<i64>) {
        self.load_chunks_range(origin_chunk_pos, self.rules.generation_distance);
    }
    fn load_chunks_range(&mut self, origin_chunk_pos: Vec2<i64>, generation_distance: usize) {
        let gen_dist = generation_distance as i64;
        for y in -gen_dist..gen_dist + 1 {
            for x in -gen_dist..gen_dist + 1 {
                let chunk_pos = vec2(x, y) + origin_chunk_pos;
                match Chunk::load(&self.world_name, chunk_pos) {
                    Ok(chunk) => {
                        for (&item_id, item) in &chunk.items {
                            self.items.insert(item_id, item.clone());
                        }
                        self.loaded_chunks.insert(chunk_pos, chunk);
                    }
                    Err(_) => {
                        if let Some(chunk) = self.loaded_chunks.get_mut(&chunk_pos) {
                            chunk.is_loaded = true;
                        } else {
                            self.generate_chunk(chunk_pos);
                        }
                    }
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
        self.loaded_chunks.insert(
            chunk_pos,
            Chunk {
                tile_map,
                items: HashMap::new(),
                is_loaded: true,
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
        match self.loaded_chunks.get(&chunk_pos) {
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
    fn get_spawnable_pos(
        &self,
        origin_chunk_pos: Vec2<i64>,
        chunk_search_range: usize,
    ) -> Option<Vec2<i64>> {
        let mut positions = vec![];
        let mut chunks = Vec::with_capacity(chunk_search_range * chunk_search_range * 4);
        let chunk_search_range = chunk_search_range as i64;
        for y in -chunk_search_range..chunk_search_range + 1 {
            for x in -chunk_search_range..chunk_search_range + 1 {
                let pos = vec2(x, y) + origin_chunk_pos;
                if let Some(_) = self.loaded_chunks.get(&pos) {
                    chunks.push(pos);
                }
            }
        }
        for chunk_pos in chunks {
            for y in 0..self.chunk_size.y as i64 {
                for x in 0..self.chunk_size.x as i64 {
                    let pos = Self::local_to_global_pos(self.chunk_size, chunk_pos, vec2(x, y));
                    if self.is_spawnable_tile(pos) {
                        positions.push(pos);
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
