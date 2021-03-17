use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePack {
    pub biome_properties: HashMap<Biome, BiomeProperties>,
    pub biome_generation: HashMap<Biome, BiomeGeneration>,
    pub world_parameters: HashMap<WorldParameter, MultiNoiseProperties>,
    pub entity_components: HashMap<EntityType, EntityComponents>,
    pub entity_generation: HashMap<Biome, Vec<ItemGeneration>>,
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

        fn load_or_default<T: Default + for<'de> Deserialize<'de>>(
            path: impl AsRef<std::path::Path>,
        ) -> std::io::Result<T> {
            match std::fs::File::open(path.as_ref()) {
                Ok(file) => {
                    let reader = std::io::BufReader::new(file);
                    Ok(serde_json::from_reader(reader)?)
                }
                Err(err) => match err.kind() {
                    std::io::ErrorKind::NotFound => Ok(T::default()),
                    _ => Err(err),
                },
            }
        }

        Ok(Self {
            biome_properties: load_or_default(server_path.join("biomes.json"))?,
            biome_generation: load_or_default(server_path.join("generation-biomes.json"))?,
            world_parameters: load_or_default(server_path.join("world-parameters.json"))?,
            entity_generation: load_or_default(server_path.join("generation-entities.json"))?,
            recipes: load_or_default(server_path.join("recipes.json"))?,
            entity_components: load_or_default(server_path.join("entities.json"))?,
        })
    }
    pub fn empty() -> Self {
        Self {
            biome_properties: HashMap::new(),
            biome_generation: HashMap::new(),
            world_parameters: HashMap::new(),
            entity_components: HashMap::new(),
            entity_generation: HashMap::new(),
            recipes: Vec::new(),
        }
    }
    pub fn merge(&mut self, resource_pack: ResourcePack) {
        self.biome_properties.extend(resource_pack.biome_properties);
        self.biome_generation.extend(resource_pack.biome_generation);
        self.world_parameters.extend(resource_pack.world_parameters);
        self.entity_components
            .extend(resource_pack.entity_components);
        self.entity_generation
            .extend(resource_pack.entity_generation);
        self.recipes.extend(resource_pack.recipes);
    }
}
