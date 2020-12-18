use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Trans)]
pub struct ResourcePack {
    pub biome_names: HashMap<String, Biome>,
    pub biomes: HashMap<Biome, BiomeGeneration>,
    pub parameters: HashMap<GenerationParameter, NoiseParameters>,
    pub items: HashMap<ItemType, ItemParameters>,
    pub item_generation: HashMap<Biome, Vec<ItemGeneration>>,
    pub recipes: Vec<Recipe>,
}

impl ResourcePack {
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
