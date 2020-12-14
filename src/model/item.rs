use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Item {
    pub pos: Vec2<f32>,
    pub size: f32,
    pub item_type: ItemType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Hash)]
pub struct ItemType(String);

impl ItemType {
    pub fn from_string(name: String) -> Self {
        Self(name)
    }
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Trans)]
pub struct ItemParameters {
    pub size: f32,
    pub traversable: bool,
    pub pickable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Trans)]
pub struct ItemGeneration {
    pub item_type: Option<ItemType>,
    pub weight: usize,
}
