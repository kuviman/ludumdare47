use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePack {
    pub biomes: HashMap<Biome, BiomeParameters>,
    pub biome_generation: HashMap<Biome, BiomeGeneration>,
    pub parameters: HashMap<GenerationParameter, MultiNoiseParameters>,
    pub items: HashMap<ItemType, ItemParameters>,
    pub item_generation: HashMap<Biome, Vec<ItemGeneration>>,
    pub recipes: Vec<Recipe>,
}

impl ResourcePack {
    pub fn load_resource_packs() -> Result<(Vec<String>, Self), std::io::Error> {
        let mut packs = Vec::new();
        let mut resource_pack = Self::empty();
        for pack in std::fs::read_dir("packs/")? {
            let pack = pack?;
            packs.push(pack.file_name().to_str().unwrap().to_owned());
            resource_pack.merge(Self::load_resource_pack(pack)?);
        }
        Ok((packs, resource_pack))
    }
    fn load_resource_pack(path: std::fs::DirEntry) -> Result<Self, std::io::Error> {
        // Load generation parameters
        let generation_parameters: HashMap<GenerationParameter, MultiNoiseParameters> =
            match std::fs::File::open(path.path().join("server/generation-parameters.json")) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load biomes
        let biomes: HashMap<Biome, BiomeParameters> =
            match std::fs::File::open(path.path().join("server/biomes.json")) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load biome generaion
        let biome_generation: HashMap<Biome, BiomeGeneration> =
            match std::fs::File::open(path.path().join("server/generation-biomes.json")) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load items
        let items: HashMap<ItemType, ItemParameters> =
            match std::fs::File::open(path.path().join("server/items.json")) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load item generation
        let item_generation: HashMap<Biome, Vec<ItemGeneration>> =
            match std::fs::File::open(path.path().join("server/generation-items.json")) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load recipes
        let recipes: Vec<Recipe> =
            match std::fs::File::open(path.path().join("server/recipes.json")) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => Vec::new(),
            };

        Ok(Self {
            biomes,
            biome_generation,
            parameters: generation_parameters,
            item_generation,
            recipes,
            items,
        })
    }
    pub fn empty() -> Self {
        Self {
            biomes: HashMap::new(),
            biome_generation: HashMap::new(),
            parameters: HashMap::new(),
            items: HashMap::new(),
            item_generation: HashMap::new(),
            recipes: Vec::new(),
        }
    }
    pub fn merge(&mut self, resource_pack: ResourcePack) {
        self.biomes.extend(resource_pack.biomes);
        self.biome_generation.extend(resource_pack.biome_generation);
        self.parameters.extend(resource_pack.parameters);
        self.items.extend(resource_pack.items);
        self.item_generation.extend(resource_pack.item_generation);
        self.recipes.extend(resource_pack.recipes);
    }
}
