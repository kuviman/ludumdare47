use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Trans, Hash, PartialEq, Eq)]
pub enum Biome {
    Water,
    Beach,
    Forest,
    Hills,
    MagicForest,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
    pub height: f32,
    pub biome: Biome,
}
