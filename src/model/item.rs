use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Item {
    pub pos: Vec2<f32>,
    pub size: f32,
    pub item_type: ItemType,
}

impl Item {
    pub fn center(&self) -> Vec2<f32> {
        self.pos.map(|x| x as f32) + vec2(self.size / 2.0, self.size / 2.0)
    }
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
            Tree | Campfire | Raft | Rock | GoldRock | MagicCrystal | BigMushroom | Statue => false,
        }
    }
    pub fn is_pickable(&self) -> bool {
        use ItemType::*;
        match self {
            Pebble | SharpStone | Stick | Axe | DoubleStick | Log | Planks | Torch | Shovel
            | Pickaxe | GoldPickaxe | GoldNugget | CrystalShard | TreasureChest => true,
            Tree | Campfire | Raft | Rock | GoldRock | MagicCrystal | BigMushroom | Statue
            | TreasureMark => false,
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
    Raft,
    Rock,
    GoldRock,
    MagicCrystal,
    BigMushroom,
    Statue,
}
