use super::*;
use noise::NoiseFn;

pub struct GenerationNoise {
    pub noise: Box<dyn NoiseFn<[f64; 2]> + Sync + Send>,
    pub parameters: NoiseParameters,
}

impl GenerationNoise {
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
    pub fn max_delta(&self) -> f32 {
        self.parameters.max_value - self.parameters.min_value
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NoiseParameters {
    pub min_value: f32,
    pub max_value: f32,
    pub scale: f32,
    pub octaves: usize,
    pub lacunarity: f32,
    pub persistance: f32,
}
