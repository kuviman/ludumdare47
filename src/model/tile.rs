use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
    pub pos: Vec2<i64>,
    pub height: f32,
    pub biome: Biome,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Trans, Hash, PartialEq, Eq)]
pub enum Biome {
    Void,
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
    pub offset: f32,
    pub parameters: HashMap<BiomeParameter, (f32, f32)>,
}

impl BiomeGeneration {
    pub fn new(offset: f32, parameters: HashMap<BiomeParameter, (f32, f32)>) -> Self {
        // TODO: check every parameter to be in range -1..1 and offset to be in range 0..1
        Self { offset, parameters }
    }
    pub fn get_distance(&self, pos: Vec2<f32>, parameter: &BiomeParameter, noise: &Noise) -> f32 {
        match self.parameters.get(parameter) {
            Some(parameter_zone) => {
                let noise_value = noise.get(pos);
                (noise_value - parameter_zone.0).min(parameter_zone.1 - noise_value)
            }
            None => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BiomeParameter {
    Height,
    Magic,
    Humidity,
}

pub struct Noise {
    pub noise: Box<dyn NoiseFn<[f64; 2]> + Sync + Send>,
    pub parameters: NoiseParameters,
}

impl Noise {
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

#[derive(Debug, Clone, Copy)]
pub struct NoiseParameters {
    pub min_value: f32,
    pub max_value: f32,
    pub scale: f32,
    pub octaves: usize,
    pub lacunarity: f32,
    pub persistance: f32,
}

impl NoiseParameters {
    pub fn new(
        min_value: f32,
        max_value: f32,
        scale: f32,
        octaves: usize,
        lacunarity: f32,
        persistance: f32,
    ) -> Self {
        assert!(scale > 0.0, "Noise scale must be positive");
        assert!(octaves > 0, "There must be at least one octave");
        assert!(lacunarity >= 1.0, "Lacunarity must be more than 1.0");
        assert!(
            persistance > 0.0,
            "Persistance must be positive and less than 1.0"
        );
        assert!(
            persistance <= 1.0,
            "Persistance must be positive and less than 1.0"
        );
        Self {
            min_value,
            max_value,
            scale,
            octaves,
            lacunarity,
            persistance,
        }
    }
}
