use super::*;

pub struct MultiNoise {
    noise: Box<dyn ::noise::NoiseFn<[f64; 2]> + Sync + Send>,
    properties: MultiNoiseProperties,
}

impl MultiNoise {
    pub fn new(seed: u32, properties: &MultiNoiseProperties) -> Self {
        use ::noise::Seedable;
        Self {
            noise: Box::new(::noise::OpenSimplex::new().set_seed(seed)),
            properties: properties.clone(),
        }
    }
    pub fn get(&self, pos: Vec2<f32>) -> f32 {
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut value = 0.0;
        for _ in 0..self.properties.octaves {
            value += self.noise.get([
                pos.x as f64 / self.properties.scale as f64 * frequency as f64,
                pos.y as f64 / self.properties.scale as f64 * frequency as f64,
            ]) as f32
                / 0.544
                * amplitude;
            frequency *= self.properties.lacunarity;
            amplitude *= self.properties.persistance;
        }
        let value = value.max(-1.0).min(1.0);
        (value / 2.0 + 0.5) * (self.properties.max_value - self.properties.min_value)
            + self.properties.min_value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiNoiseProperties {
    pub min_value: f32,
    pub max_value: f32,
    pub scale: f32,
    pub octaves: usize,
    pub lacunarity: f32,
    pub persistance: f32,
}
