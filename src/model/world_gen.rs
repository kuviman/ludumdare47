use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeGeneration {
    pub world_parameters: HashMap<WorldParameter, (f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemGeneration {
    pub entity_type: Option<EntityType>,
    pub weight: usize,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldParameter(pub String);

pub struct WorldGen {
    world_parameters: HashMap<WorldParameter, MultiNoise>,
    biome_generation: HashMap<Biome, BiomeGeneration>,
    entity_generation: HashMap<Biome, Vec<ItemGeneration>>,
    pub entity_properties: HashMap<EntityType, EntityProperties>,
}

impl WorldGen {
    pub fn new(seed: u32, resource_pack: &ResourcePack) -> Self {
        let seed_noise = ::noise::OpenSimplex::new().set_seed(seed);
        Self {
            world_parameters: resource_pack
                .world_parameters
                .iter()
                .map(|(parameter, multi_noise_properties)| {
                    fn hash<T>(obj: T) -> u32
                    where
                        T: std::hash::Hash,
                    {
                        use std::hash::*;
                        let mut hasher = siphasher::sip::SipHasher::new();
                        obj.hash(&mut hasher);
                        hasher.finish() as u32
                    }
                    (
                        parameter.clone(),
                        MultiNoise::new(
                            (seed_noise.get([hash(parameter) as f64, 0.0]) * 1e5).abs() as u32,
                            multi_noise_properties,
                        ),
                    )
                })
                .collect(),
            biome_generation: resource_pack.biome_generation.clone(),
            entity_generation: resource_pack.entity_generation.clone(),
            entity_properties: resource_pack.entity_properties.clone(),
        }
    }
    pub fn generate_tile(&self, pos: Vec2<i64>) -> (Tile, Option<EntityType>) {
        let world_parameters: HashMap<WorldParameter, f32> = self
            .world_parameters
            .iter()
            .map(|(parameter, multi_noise)| {
                (
                    parameter.clone(),
                    multi_noise.get(pos.map(|x| x as f32 + 0.5)),
                )
            })
            .collect();
        let biome = self
            .biome_generation
            .iter()
            .filter_map(|(biome, biome_gen)| {
                let mut total_score = 0.0;
                for (world_parameter, zone) in &biome_gen.world_parameters {
                    let world_parameter_value = world_parameters[world_parameter];
                    let world_parameter_score =
                        (world_parameter_value - zone.0).min(zone.1 - world_parameter_value);
                    if world_parameter_score < 0.0 {
                        return None;
                    } else {
                        total_score += world_parameter_score;
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
            world_parameters,
        };

        let entity_type = self
            .entity_generation
            .get(&biome)
            .map(|gen| {
                gen.choose_weighted(&mut global_rng(), |item| item.weight)
                    .unwrap()
            })
            .map(|item| item.entity_type.clone())
            .flatten();
        (tile, entity_type)
    }
}
