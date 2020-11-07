use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
    pub pos: Vec2<i64>,
    pub height: f32,
    pub biome: Biome,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Trans, Hash, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Island,
    Beach,
    Lake,
    Forest,
    Hills,
    MagicForest,
}

#[derive(Debug, Clone)]
pub struct BiomeGeneration {
    pub parent_biome: Option<Biome>,
    pub parameters: HashMap<BiomeParameters, f32>,
}

impl BiomeGeneration {
    pub fn new(parent_biome: Option<Biome>, parameters: HashMap<BiomeParameters, f32>) -> Self {
        Self {
            parent_biome,
            parameters,
        }
    }
    pub fn calculate_score(
        &self,
        pos: Vec2<i64>,
        noises: &HashMap<BiomeParameters, (Box<dyn NoiseFn<[f64; 2]>>, NoiseParameters)>,
    ) -> f32 {
        let mut score = 0.0;
        for (parameter, &parameter_value) in &self.parameters {
            let (noise, noise_parameters) = &noises[parameter];
            let noise = noise.get([
                pos.x as f64 / noise_parameters.scale as f64,
                pos.y as f64 / noise_parameters.scale as f64,
            ]) as f32;
            score += 2.0 - (parameter_value - noise).abs();
        }
        score
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BiomeParameters {
    Height,
    Temperature,
    Humidity,
}

#[derive(Debug, Clone, Copy)]
pub struct NoiseParameters {
    pub scale: f32,
}

impl NoiseParameters {
    pub fn new(scale: f32) -> Self {
        assert!(scale > 0.0, "Noise scale must be positive");
        Self { scale }
    }
}
