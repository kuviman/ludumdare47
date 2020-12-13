use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ticks_per_second: f32,
    pub chunk_size: Vec2<usize>,
    pub initial_generation_size: Vec2<usize>,
    pub player_movement_speed: f32,
    pub view_distance: f32,
    pub regeneration_percent: f32,
    pub campfire_light: f32,
    pub torch_light: f32,
    pub statue_light: f32,
    pub sound_distance: f32,
    pub entity_interaction_range: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ticks_per_second: 20.0,
            chunk_size: vec2(10, 10),
            initial_generation_size: vec2(20, 20),
            player_movement_speed: 2.0,
            view_distance: 20.0,
            regeneration_percent: 0.01,
            campfire_light: 5.0,
            torch_light: 5.0,
            statue_light: 10.0,
            sound_distance: 5.0,
            entity_interaction_range: 1.5,
        }
    }
}

impl Config {
    pub fn load_resource_packs() -> Result<(Vec<String>, ResourcePack), std::io::Error> {
        let mut packs = Vec::new();
        let mut resource_pack = ResourcePack::empty();
        for pack in std::fs::read_dir("packs/")? {
            let pack = pack?;
            packs.push(pack.file_name().to_str().unwrap().to_owned());
            resource_pack.merge(Self::load_resource_pack(pack)?);
        }
        Ok((packs, resource_pack))
    }
    fn load_resource_pack(path: std::fs::DirEntry) -> Result<ResourcePack, std::io::Error> {
        let parameters_path = path.path().join("server/generation-parameters.json");
        let generation_parameters: HashMap<BiomeParameter, NoiseParameters> =
            serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(
                parameters_path,
            )?))?;

        let biomes_path = path.path().join("server/generation-biomes.json");
        let biomes: HashMap<String, BiomeGeneration> =
            serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(biomes_path)?))?;

        let mut biome_names = HashMap::with_capacity(biomes.len());
        let mut biome_gen = HashMap::with_capacity(biomes.len());
        for (biome_name, biome_generation) in biomes {
            let biome = Biome::new(biome_name.clone());
            biome_names.insert(biome_name, biome.clone());
            biome_gen.insert(biome, biome_generation);
        }

        let recipes_path = path.path().join("server/recipes.json");
        let recipes: Vec<Recipe> =
            serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(recipes_path)?))?;

        Ok(ResourcePack {
            biome_names,
            biomes: biome_gen,
            parameters: generation_parameters,
            recipes,
        })
    }
    pub fn default_generation_choices(
        resource_pack: &ResourcePack,
    ) -> HashMap<Biome, Vec<(Option<ItemType>, usize)>> {
        let mut generation_choices = HashMap::new();
        generation_choices.insert(
            resource_pack.get_biome("Ocean").unwrap().clone(),
            vec![(None, 1)],
        );
        generation_choices.insert(
            resource_pack.get_biome("Lake").unwrap().clone(),
            vec![(None, 1)],
        );
        generation_choices.insert(
            resource_pack.get_biome("Beach").unwrap().clone(),
            vec![(None, 200), (Some(ItemType::TreasureMark), 1)],
        );
        generation_choices.insert(
            resource_pack.get_biome("Forest").unwrap().clone(),
            vec![
                (None, 300),
                (Some(ItemType::Tree), 30),
                (Some(ItemType::Stick), 10),
            ],
        );
        generation_choices.insert(
            resource_pack.get_biome("Hills").unwrap().clone(),
            vec![
                (None, 300),
                ((Some(ItemType::Pebble)), 20),
                ((Some(ItemType::Rock)), 10),
                ((Some(ItemType::GoldRock)), 1),
            ],
        );
        generation_choices.insert(
            resource_pack.get_biome("MagicForest").unwrap().clone(),
            vec![
                (None, 300),
                ((Some(ItemType::BigMushroom)), 10),
                ((Some(ItemType::MagicCrystal)), 1),
            ],
        );
        generation_choices
    }
    pub fn default_scores_map() -> HashMap<ItemType, i32> {
        let mut scores_map = HashMap::new();
        scores_map.insert(ItemType::TreasureChest, 5);
        scores_map.insert(ItemType::CrystalShard, 10);
        scores_map.insert(ItemType::GoldNugget, 5);
        scores_map.insert(ItemType::Stick, -1);
        scores_map.insert(ItemType::Pebble, -1);
        scores_map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Trans)]
pub struct ResourcePack {
    pub biome_names: HashMap<String, Biome>,
    pub biomes: HashMap<Biome, BiomeGeneration>,
    pub parameters: HashMap<BiomeParameter, NoiseParameters>,
    pub recipes: Vec<Recipe>,
}

impl ResourcePack {
    pub fn empty() -> Self {
        Self {
            biome_names: HashMap::new(),
            biomes: HashMap::new(),
            parameters: HashMap::new(),
            recipes: Vec::new(),
        }
    }
    pub fn merge(&mut self, resource_pack: ResourcePack) {
        self.biome_names.extend(resource_pack.biome_names);
        self.biomes.extend(resource_pack.biomes);
        self.parameters.extend(resource_pack.parameters);
        self.recipes.extend(resource_pack.recipes);
    }
    pub fn get_biome(&self, biome_name: &str) -> Option<&Biome> {
        self.biome_names.get(biome_name)
    }
}
