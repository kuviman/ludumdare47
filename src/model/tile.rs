use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Trans, Hash, PartialEq, Eq)]
pub enum GroundType {
    Water,
    Sand,
    Grass,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
    pub pos: Vec2<usize>,
    pub ground_type: GroundType,
}
