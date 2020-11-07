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
    Beach,
    Lake,
    Forest,
    Hills,
    MagicForest,
}

impl Biome {
    pub fn height(&self) -> f32 {
        match self {
            Self::Ocean => -1.0,
            Self::Lake => -0.2,
            _ => 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiomeGeneration {
    pub parameters: HashMap<BiomeParameters, f32>,
}

impl BiomeGeneration {
    pub fn new(parameters: HashMap<BiomeParameters, f32>) -> Self {
        Self { parameters }
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
        score / self.parameters.len() as f32
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
