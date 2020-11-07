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
        biome: Biome,
        pos: Vec2<i64>,
        noises: &HashMap<BiomeParameters, &dyn NoiseFn<[f64; 2]>>,
    ) -> f32 {
        match biome {
            Biome::Ocean | Biome::Island => Self::calculate_parameter(
                pos,
                &BiomeParameters::Height,
                self.parameters[&BiomeParameters::Height],
                noises,
            ),
            _ => {
                let mut score = 0.0;
                for (parameter, &value) in &self.parameters {
                    score += Self::calculate_parameter(pos, parameter, value, noises);
                }
                score
            }
        }
    }
    fn calculate_parameter(
        pos: Vec2<i64>,
        parameter: &BiomeParameters,
        parameter_value: f32,
        noises: &HashMap<BiomeParameters, &dyn NoiseFn<[f64; 2]>>,
    ) -> f32 {
        let noise = noises[parameter].get([pos.x as f64 / 20.0, pos.y as f64 / 20.0]) as f32;
        2.0 - (parameter_value - noise).abs()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BiomeParameters {
    Height,
    Temperature,
    Humidity,
}
