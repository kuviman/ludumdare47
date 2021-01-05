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
    pub fn load_all(
        path: impl AsRef<std::path::Path>,
    ) -> Result<(Vec<String>, Self), std::io::Error> {
        let mut packs = Vec::new();
        let mut resource_pack = Self::empty();
        for pack in std::fs::read_dir(path.as_ref())? {
            let pack = pack?;
            packs.push(pack.file_name().to_str().unwrap().to_owned());
            resource_pack.merge(Self::load(pack.path())?);
        }
        Ok((packs, resource_pack))
    }
    fn load(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let server_path = path.join("server");

        fn load<T: for<'de> Deserialize<'de>>(
            path: impl AsRef<std::path::Path>,
        ) -> std::io::Result<T> {
            let file = std::fs::File::open(path.as_ref())?;
            let reader = std::io::BufReader::new(file);
            Ok(serde_json::from_reader(reader)?)
        }

        Ok(Self {
            biomes: load(server_path.join("biomes.json"))?,
            biome_generation: load(server_path.join("generation-biomes.json"))?,
            parameters: load(server_path.join("generation-parameters.json"))?,
            item_generation: load(server_path.join("generation-items.json"))?,
            recipes: load(server_path.join("recipes.json"))?,
            items: load(server_path.join("items.json"))?,
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
