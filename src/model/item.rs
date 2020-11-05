use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Item {
    pub pos: Vec2<f32>,
    pub size: f32,
    pub item_type: ItemType,
}

impl ItemType {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
    pub fn is_traversable(&self) -> bool {
        use ItemType::*;
        match self {
            Pebble | SharpStone | Stick | Axe | DoubleStick | Log | Planks | Torch | Shovel
            | Pickaxe | GoldPickaxe | GoldNugget | CrystalShard | TreasureMark | TreasureChest => {
                true
            }
            Tree | Campfire | Rock | GoldRock | MagicCrystal | BigMushroom | Statue => false,
        }
    }
    pub fn is_pickable(&self) -> bool {
        use ItemType::*;
        match self {
            Pebble | SharpStone | Stick | Axe | DoubleStick | Log | Planks | Torch | Shovel
            | Pickaxe | GoldPickaxe | GoldNugget | CrystalShard | TreasureChest => true,
            Tree | Campfire | Rock | GoldRock | MagicCrystal | BigMushroom | Statue
            | TreasureMark => false,
        }
    }
    pub fn size(&self) -> f32 {
        use ItemType::*;
        match self {
            Pebble | SharpStone | Stick | Axe | DoubleStick | Log | Planks | Torch | Shovel
            | Pickaxe | GoldPickaxe | GoldNugget | CrystalShard | TreasureMark | TreasureChest => {
                0.5
            }
            Tree | Campfire | Rock | GoldRock | MagicCrystal | BigMushroom | Statue => 0.5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Copy, Hash)]
pub enum ItemType {
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
    Tree,
    Campfire,
    Rock,
    GoldRock,
    MagicCrystal,
    BigMushroom,
    Statue,
}
