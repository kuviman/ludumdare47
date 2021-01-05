use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: Id,
    pub pos: Vec2<f32>,
    pub item_type: ItemType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ItemType(String);

impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ItemParameters {
    pub size: f32,
    pub traversable: bool,
    pub pickable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemGeneration {
    pub item_type: Option<ItemType>,
    pub weight: usize,
}
