use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
    pub pos: Vec2<i64>,
    pub biome: Biome,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Trans, Hash, PartialEq, Eq)]
pub enum Biome {
    Beach,
    Lake,
    Forest,
    Hills,
    MagicForest,
}

#[derive(Debug, Copy, Clone)]
pub struct BiomeGeneration {
    pub height: f32,
    pub size: f32,
    pub weight: f32,
}

impl BiomeGeneration {
    pub fn new(height: f32, size: f32, weight: f32) -> Self {
        assert!(
            size >= 0.0,
            "Size in BiomeGeneration must be in range 0.0..1.0"
        );
        assert!(
            size <= 1.0,
            "Size in BiomeGeneration must be in range 0.0..1.0"
        );
        Self {
            height,
            size,
            weight,
        }
    }
}
