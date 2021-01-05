use super::*;

pub struct WorldGen {
    parameters: HashMap<GenerationParameter, GenerationNoise>,
    biomes: HashMap<Biome, BiomeGeneration>,
    item_generation: HashMap<Biome, Vec<ItemGeneration>>,
    item_parameters: HashMap<ItemType, ItemParameters>,
}

impl WorldGen {
    pub fn new(seed: u32, resource_pack: &ResourcePack) -> Self {
        let seed_noise = OpenSimplex::new().set_seed(seed);
        Self {
            parameters: resource_pack
                .parameters
                .iter()
                .map(|(parameter, noise_parameters)| {
                    fn hash<T>(obj: T) -> u64
                    where
                        T: std::hash::Hash,
                    {
                        use std::hash::*;
                        let mut hasher = siphasher::sip::SipHasher::new();
                        obj.hash(&mut hasher);
                        hasher.finish()
                    }
                    (
                        parameter.clone(),
                        GenerationNoise {
                            noise: Box::new(OpenSimplex::new().set_seed(
                                (seed_noise.get([hash(parameter) as f64, 0.0]) * 1000.0) as u32,
                            )),
                            parameters: noise_parameters.clone(),
                        },
                    )
                })
                .collect(),
            biomes: resource_pack.biomes.clone(),
            item_generation: resource_pack.item_generation.clone(),
            item_parameters: resource_pack.items.clone(),
        }
    }
    pub fn generate_tile(
        &self,
        id_generator: &mut IdGenerator,
        pos: Vec2<i64>,
    ) -> (Tile, Option<Item>) {
        let parameters: HashMap<GenerationParameter, f32> = self
            .parameters
            .iter()
            .map(|(parameter, noise)| (parameter.clone(), noise.get(pos.map(|x| x as f32 + 0.5))))
            .collect();
        let biome = self
            .biomes
            .iter()
            .filter_map(|(biome, biome_gen)| {
                let mut total_score = 0.0;
                for (parameter, zone) in &biome_gen.parameters {
                    let value = parameters[parameter];
                    let parameter_score = (value - zone.0).min(zone.1 - value);
                    if parameter_score < 0.0 {
                        return None;
                    } else {
                        total_score += parameter_score;
                    }
                }
                Some((biome, total_score))
            })
            .min_by_key(|(_, score)| r32(*score))
            .unwrap()
            .0
            .clone();

        let tile = Tile {
            biome: biome.clone(),
            parameters,
        };

        let item = self
            .item_generation
            .get(&biome)
            .map(|gen| {
                gen.choose_weighted(&mut global_rng(), |item| item.weight)
                    .unwrap()
            })
            .map(|item| item.item_type.as_ref())
            .flatten()
            .map(|item_type| Item {
                id: id_generator.gen(),
                pos: pos.map(|x| x as f32 + 0.5),
                size: self.item_parameters[item_type].size,
                item_type: item_type.clone(),
            });
        (tile, item)
    }
}
