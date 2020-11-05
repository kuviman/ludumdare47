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
    pub pos: Vec2<i64>,
    pub biome: Biome,
}
