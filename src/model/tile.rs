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

#[derive(Debug, Clone)]
pub struct BiomeGeneration {
    pub parent_biome: Option<Biome>,
    pub height: f32,
    pub parameters: HashMap<BiomeParameters, f32>,
}

impl BiomeGeneration {
    pub fn new(
        parent_biome: Option<Biome>,
        height: f32,
        parameters: HashMap<BiomeParameters, f32>,
    ) -> Self {
        Self {
            parent_biome,
            height,
            parameters,
        }
    }
    pub fn calculate_score(
        &self,
        pos: Vec2<i64>,
        noises: &HashMap<BiomeParameters, &dyn NoiseFn<[f64; 2]>>,
    ) -> f32 {
        let mut score = 0.0;
        for (parameters, noise) in noises {
            let value = noise.get([pos.x as f64 / 20.0, pos.y as f64 / 20.0]) as f32;
            let parameter = self.parameters[parameters];
            score -= (value - parameter).abs();
        }
        score
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BiomeParameters {
    Temperature,
    Humidity,
}
