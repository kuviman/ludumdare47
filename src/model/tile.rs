use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile {
    pub biome: Biome,
    pub parameters: HashMap<GenerationParameter, f32>,
}
