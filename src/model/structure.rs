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
    Rock,
    GoldRock,
    MagicCrystal,
    BigMushroom,
    Statue,
}

impl StructureType {
    pub fn traversable(&self) -> bool {
        match self {
            Self::Item { .. } => true,
            _ => false,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Self::Item { item } => item.to_string(),
            _ => format!("{:?}", self),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Copy, Hash)]
pub enum Item {
    Pebble,
    SharpStone,
    Stick,
    Axe,
    DoubleStick,
    Log,
    Planks,
    Torch,
    Shovel,
    Pickaxe,
    GoldPickaxe,
    GoldNugget,
    CrystalShard,
    TreasureMark,
    TreasureChest,
}

impl Item {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
