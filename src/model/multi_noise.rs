use super::*;

pub struct MultiNoise {
    noise: Box<dyn ::noise::NoiseFn<[f64; 2]> + Sync + Send>,
    parameters: MultiNoiseParameters,
}

impl MultiNoise {
    pub fn new(seed: u32, parameters: &MultiNoiseParameters) -> Self {
        use ::noise::Seedable;
        Self {
            noise: Box::new(::noise::OpenSimplex::new().set_seed(seed)),
            parameters: parameters.clone(),
        }
    }
    pub fn get(&self, pos: Vec2<f32>) -> f32 {
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut value = 0.0;
        for _ in 0..self.parameters.octaves {
            value += self.noise.get([
                pos.x as f64 / self.parameters.scale as f64 * frequency as f64,
                pos.y as f64 / self.parameters.scale as f64 * frequency as f64,
            ]) as f32
                / 0.544
                * amplitude;
            frequency *= self.parameters.lacunarity;
            amplitude *= self.parameters.persistance;
        }
        let value = value.max(-1.0).min(1.0);
        (value / 2.0 + 0.5) * (self.parameters.max_value - self.parameters.min_value)
            + self.parameters.min_value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiNoiseParameters {
    pub min_value: f32,
    pub max_value: f32,
    pub scale: f32,
    pub octaves: usize,
    pub lacunarity: f32,
    pub persistance: f32,
}
