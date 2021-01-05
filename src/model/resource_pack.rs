use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePack {
    pub biome_properties: HashMap<Biome, BiomeProperties>,
    pub biome_generation: HashMap<Biome, BiomeGeneration>,
    pub world_parameters: HashMap<WorldParameter, MultiNoiseProperties>,
    pub item_properties: HashMap<ItemType, ItemProperties>,
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
            biome_properties: load(server_path.join("biomes.json"))?,
            biome_generation: load(server_path.join("generation-biomes.json"))?,
            world_parameters: load(server_path.join("world-parameters.json"))?,
            item_generation: load(server_path.join("generation-items.json"))?,
            recipes: load(server_path.join("recipes.json"))?,
            item_properties: load(server_path.join("items.json"))?,
        })
    }
    pub fn empty() -> Self {
        Self {
            biome_properties: HashMap::new(),
            biome_generation: HashMap::new(),
            world_parameters: HashMap::new(),
            item_properties: HashMap::new(),
            item_generation: HashMap::new(),
            recipes: Vec::new(),
        }
    }
    pub fn merge(&mut self, resource_pack: ResourcePack) {
        self.biome_properties.extend(resource_pack.biome_properties);
        self.biome_generation.extend(resource_pack.biome_generation);
        self.world_parameters.extend(resource_pack.world_parameters);
        self.item_properties.extend(resource_pack.item_properties);
        self.item_generation.extend(resource_pack.item_generation);
        self.recipes.extend(resource_pack.recipes);
    }
}
