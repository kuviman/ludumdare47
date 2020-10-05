use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ticks_per_second: f32,
    pub map_size: Vec2<usize>,
    pub player_day_view_distance: f32,
    pub player_night_view_distance: f32,
    pub day_length: usize,
    pub night_length: usize,
    pub fire_extinguish_chance: f32,
    pub regeneration_percent: f32,
    pub campfire_light: f32,
    pub torch_light: f32,
    pub statue_light: f32,
    pub sound_distance: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ticks_per_second: 2.0,
            map_size: vec2(256, 256),
            player_day_view_distance: 10.0,
            player_night_view_distance: 3.0,
            day_length: 100,
            night_length: 50,
            fire_extinguish_chance: 0.001,
            regeneration_percent: 0.01,
            campfire_light: 5.0,
            torch_light: 5.0,
            statue_light: 10.0,
            sound_distance: 5.0,
        }
    }
}

impl Config {
    pub fn default_recipes() -> Vec<Recipe> {
        vec![
            Recipe {
                ingredient1: Some(Item::Pebble),
                ingredient2: Some(StructureType::Item { item: Item::Pebble }),
                result1: None,
                result2: Some(StructureType::Item {
                    item: Item::SharpStone,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Pebble),
                ingredient2: Some(StructureType::Rock),
                result1: Some(Item::SharpStone),
                result2: Some(StructureType::Rock),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Stick),
                ingredient2: Some(StructureType::Item { item: Item::Pebble }),
                result1: Some(Item::Shovel),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Pebble),
                ingredient2: Some(StructureType::Item { item: Item::Stick }),
                result1: Some(Item::Shovel),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Stick),
                ingredient2: Some(StructureType::Item {
                    item: Item::SharpStone,
                }),
                result1: Some(Item::Axe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::SharpStone),
                ingredient2: Some(StructureType::Item { item: Item::Stick }),
                result1: Some(Item::Axe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Axe),
                ingredient2: Some(StructureType::Item {
                    item: Item::SharpStone,
                }),
                result1: Some(Item::Pickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::SharpStone),
                ingredient2: Some(StructureType::Item { item: Item::Axe }),
                result1: Some(Item::Pickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Pickaxe),
                ingredient2: Some(StructureType::Rock),
                result1: Some(Item::Pickaxe),
                result2: Some(StructureType::Item {
                    item: Item::SharpStone,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Pickaxe),
                ingredient2: Some(StructureType::GoldRock),
                result1: Some(Item::Pickaxe),
                result2: Some(StructureType::Item {
                    item: Item::GoldNugget,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Stick),
                ingredient2: Some(StructureType::Item {
                    item: Item::GoldNugget,
                }),
                result1: Some(Item::GoldPickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::GoldNugget),
                ingredient2: Some(StructureType::Item { item: Item::Stick }),
                result1: Some(Item::GoldPickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Stick),
                ingredient2: Some(StructureType::Item { item: Item::Stick }),
                result1: None,
                result2: Some(StructureType::Item {
                    item: Item::DoubleStick,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Axe),
                ingredient2: Some(StructureType::Tree),
                result1: Some(Item::Axe),
                result2: Some(StructureType::Item { item: Item::Log }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Axe),
                ingredient2: Some(StructureType::Item { item: Item::Log }),
                result1: Some(Item::Axe),
                result2: Some(StructureType::Item { item: Item::Planks }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Log),
                ingredient2: Some(StructureType::Item {
                    item: Item::DoubleStick,
                }),
                result1: None,
                result2: Some(StructureType::Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::DoubleStick),
                ingredient2: Some(StructureType::Item { item: Item::Log }),
                result1: None,
                result2: Some(StructureType::Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Planks),
                ingredient2: Some(StructureType::Item { item: Item::Planks }),
                result1: None,
                result2: Some(StructureType::Raft),
                conditions: Some(Biome::Water),
            },
            Recipe {
                ingredient1: Some(Item::DoubleStick),
                ingredient2: None,
                result1: Some(Item::Stick),
                result2: Some(StructureType::Item { item: Item::Stick }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Stick),
                ingredient2: Some(StructureType::Campfire),
                result1: Some(Item::Torch),
                result2: Some(StructureType::Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::GoldPickaxe),
                ingredient2: Some(StructureType::MagicCrystal),
                result1: Some(Item::GoldPickaxe),
                result2: Some(StructureType::Item {
                    item: Item::CrystalShard,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::GoldPickaxe),
                ingredient2: Some(StructureType::Rock),
                result1: Some(Item::Pickaxe),
                result2: Some(StructureType::Item {
                    item: Item::SharpStone,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::GoldPickaxe),
                ingredient2: Some(StructureType::GoldRock),
                result1: Some(Item::Pickaxe),
                result2: Some(StructureType::Item {
                    item: Item::GoldNugget,
                }),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Item::Shovel),
                ingredient2: Some(StructureType::Item {
                    item: Item::TreasureMark,
                }),
                result1: Some(Item::Shovel),
                result2: Some(StructureType::Item {
                    item: Item::TreasureChest,
                }),
                conditions: None,
            },
        ]
    }

    pub fn default_generation_choices() -> HashMap<Biome, Vec<(Option<Structure>, usize)>> {
        let basic_structure = Structure {
            pos: vec2(0, 0),
            structure_type: StructureType::Tree,
        };
        let mut generation_choices = HashMap::new();
        generation_choices.insert(Biome::Water, vec![(None, 1)]);
        generation_choices.insert(
            Biome::Beach,
            vec![
                (None, 200),
                (
                    Some(Structure {
                        structure_type: StructureType::Item {
                            item: Item::TreasureMark,
                        },
                        ..basic_structure
                    }),
                    1,
                ),
            ],
        );
        generation_choices.insert(
            Biome::Forest,
            vec![
                (None, 300),
                (Some(basic_structure.clone()), 30),
                (
                    Some(Structure {
                        structure_type: StructureType::Item { item: Item::Stick },
                        ..basic_structure
                    }),
                    10,
                ),
            ],
        );
        generation_choices.insert(
            Biome::Hills,
            vec![
                (None, 300),
                (
                    (Some(Structure {
                        structure_type: StructureType::Item { item: Item::Pebble },
                        ..basic_structure
                    })),
                    20,
                ),
                (
                    (Some(Structure {
                        structure_type: StructureType::Rock,
                        ..basic_structure
                    })),
                    10,
                ),
                (
                    (Some(Structure {
                        structure_type: StructureType::GoldRock,
                        ..basic_structure
                    })),
                    1,
                ),
            ],
        );
        generation_choices.insert(
            Biome::MagicForest,
            vec![
                (None, 300),
                (
                    (Some(Structure {
                        structure_type: StructureType::BigMushroom,
                        ..basic_structure
                    })),
                    10,
                ),
                (
                    (Some(Structure {
                        structure_type: StructureType::MagicCrystal,
                        ..basic_structure
                    })),
                    1,
                ),
            ],
        );
        generation_choices
    }
    pub fn default_scores_map() -> HashMap<Item, i32> {
        let mut scores_map = HashMap::new();
        scores_map.insert(Item::TreasureChest, 5);
        scores_map.insert(Item::CrystalShard, 10);
        scores_map.insert(Item::GoldNugget, 5);
        scores_map.insert(Item::Stick, -1);
        scores_map.insert(Item::Pebble, -1);
        scores_map
    }
}
