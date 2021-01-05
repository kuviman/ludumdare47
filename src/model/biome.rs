use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct Biome(String);

impl Biome {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl Display for Biome {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
