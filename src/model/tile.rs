use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile {
    pub biome: Biome,
    pub world_parameters: HashMap<WorldParameter, f32>,
}
