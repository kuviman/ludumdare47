use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePack {
    pub biome_names: HashMap<String, Biome>,
    pub biomes: HashMap<Biome, BiomeGeneration>,
    pub parameters: HashMap<GenerationParameter, NoiseParameters>,
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
        // Load noise maps
        let parameters_path = path.path().join("server/generation-parameters.json");
        let generation_parameters: HashMap<GenerationParameter, NoiseParameters> =
            match std::fs::File::open(parameters_path) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load biomes
        let biomes_path = path.path().join("server/generation-biomes.json");
        let mut biome_names = HashMap::new();
        let mut biome_gen = HashMap::new();
        match std::fs::File::open(biomes_path) {
            Ok(file) => {
                let biomes: HashMap<String, BiomeGeneration> =
                    serde_json::from_reader(std::io::BufReader::new(file))?;
                for (biome_name, biome_generation) in biomes {
                    let biome = Biome::new(biome_name.clone());
                    biome_names.insert(biome_name, biome.clone());
                    biome_gen.insert(biome, biome_generation);
                }
            }
            Err(_) => (),
        }

        // Load items
        let items_path = path.path().join("server/items.json");
        let items: HashMap<ItemType, ItemParameters> = match std::fs::File::open(items_path) {
            Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
            Err(_) => HashMap::new(),
        };

        // Load items generation
        let items_gen_path = path.path().join("server/generation-items.json");
        let items_gen: HashMap<Biome, Vec<ItemGeneration>> =
            match std::fs::File::open(items_gen_path) {
                Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
                Err(_) => HashMap::new(),
            };

        // Load recipes
        let recipes_path = path.path().join("server/recipes.json");
        let recipes: Vec<Recipe> = match std::fs::File::open(recipes_path) {
            Ok(file) => serde_json::from_reader(std::io::BufReader::new(file))?,
            Err(_) => Vec::new(),
        };

        Ok(Self {
            biome_names,
            biomes: biome_gen,
            parameters: generation_parameters,
            item_generation: items_gen,
            recipes,
            items,
        })
    }
    pub fn empty() -> Self {
        Self {
            biome_names: HashMap::new(),
            biomes: HashMap::new(),
            parameters: HashMap::new(),
            items: HashMap::new(),
            item_generation: HashMap::new(),
            recipes: Vec::new(),
        }
    }
    pub fn merge(&mut self, resource_pack: ResourcePack) {
        self.biome_names.extend(resource_pack.biome_names);
        self.biomes.extend(resource_pack.biomes);
        self.parameters.extend(resource_pack.parameters);
        self.items.extend(resource_pack.items);
        self.item_generation.extend(resource_pack.item_generation);
        self.recipes.extend(resource_pack.recipes);
    }
}
