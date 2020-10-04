use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Structure {
    pub pos: Vec2<usize>,
    pub structure_type: StructureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Copy, Hash)]
pub enum StructureType {
    Item { item: Item },
    Tree,
    Campfire,
    Raft,
}

impl StructureType {
    pub fn traversable(&self) -> bool {
        match self {
            Self::Item { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Copy, Hash)]
pub enum Item {
    Pebble,
    Stick,
    Axe,
    DoubleStick,
    Log,
    Planks,
    Torch,
}
