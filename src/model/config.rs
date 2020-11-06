use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ticks_per_second: f32,
    pub map_size: Vec2<usize>,
    pub player_movement_speed: f32,
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
    pub entity_interaction_range: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ticks_per_second: 20.0,
            map_size: vec2(256, 256),
            player_movement_speed: 2.0,
            player_day_view_distance: 300.0,
            player_night_view_distance: 300.0,
            day_length: 1000,
            night_length: 500,
            fire_extinguish_chance: 0.001,
            regeneration_percent: 0.01,
            campfire_light: 5.0,
            torch_light: 5.0,
            statue_light: 10.0,
            sound_distance: 5.0,
            entity_interaction_range: 1.5,
        }
    }
}

impl Config {
    pub fn default_recipes() -> Vec<Recipe> {
        use ItemType::*;
        vec![
            Recipe {
                ingredient1: Some(Pebble),
                ingredient2: Some(Pebble),
                result1: None,
                result2: Some(SharpStone),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Pebble),
                ingredient2: Some(Rock),
                result1: Some(SharpStone),
                result2: Some(Rock),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(Pebble),
                result1: Some(Shovel),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Pebble),
                ingredient2: Some(Stick),
                result1: Some(Shovel),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(SharpStone),
                result1: Some(Axe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(SharpStone),
                ingredient2: Some(Stick),
                result1: Some(Axe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Axe),
                ingredient2: Some(SharpStone),
                result1: Some(Pickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(SharpStone),
                ingredient2: Some(Axe),
                result1: Some(Pickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Pickaxe),
                ingredient2: Some(Rock),
                result1: Some(Pickaxe),
                result2: Some(SharpStone),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Pickaxe),
                ingredient2: Some(GoldRock),
                result1: Some(Pickaxe),
                result2: Some(GoldNugget),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(GoldNugget),
                result1: Some(GoldPickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(GoldNugget),
                ingredient2: Some(Stick),
                result1: Some(GoldPickaxe),
                result2: None,
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(Stick),
                result1: None,
                result2: Some(DoubleStick),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Axe),
                ingredient2: Some(Tree),
                result1: Some(Axe),
                result2: Some(Log),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Axe),
                ingredient2: Some(Log),
                result1: Some(Axe),
                result2: Some(Planks),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Log),
                ingredient2: Some(DoubleStick),
                result1: None,
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Log),
                ingredient2: Some(Planks),
                result1: None,
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Torch),
                ingredient2: Some(Log),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Torch),
                ingredient2: Some(Planks),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Torch),
                ingredient2: Some(DoubleStick),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(DoubleStick),
                ingredient2: Some(Log),
                result1: None,
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(DoubleStick),
                ingredient2: None,
                result1: Some(Stick),
                result2: Some(Stick),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(Campfire),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(GoldPickaxe),
                ingredient2: Some(MagicCrystal),
                result1: Some(GoldPickaxe),
                result2: Some(CrystalShard),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(GoldPickaxe),
                ingredient2: Some(Rock),
                result1: Some(GoldPickaxe),
                result2: Some(SharpStone),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(GoldPickaxe),
                ingredient2: Some(GoldRock),
                result1: Some(GoldPickaxe),
                result2: Some(GoldNugget),
                conditions: None,
            },
            Recipe {
                ingredient1: Some(Shovel),
                ingredient2: Some(TreasureMark),
                result1: Some(Shovel),
                result2: Some(TreasureChest),
                conditions: None,
            },
        ]
    }
    pub fn default_biomes() -> HashMap<Biome, BiomeGeneration> {
        use Biome::*;

        let mut biomes = HashMap::new();
        biomes.insert(
            Beach,
            BiomeGeneration::new(None, 0.5, {
                let mut map = HashMap::new();
                map.insert(BiomeParameters::Temperature, 0.7);
                map.insert(BiomeParameters::Humidity, -0.7);
                map
            }),
        );
        biomes.insert(
            Lake,
            BiomeGeneration::new(Some(Forest), -1.0, {
                let mut map = HashMap::new();
                map.insert(BiomeParameters::Temperature, 0.1);
                map.insert(BiomeParameters::Humidity, 0.5);
                map
            }),
        );
        biomes.insert(
            Forest,
            BiomeGeneration::new(None, 0.5, {
                let mut map = HashMap::new();
                map.insert(BiomeParameters::Temperature, -0.1);
                map.insert(BiomeParameters::Humidity, -0.1);
                map
            }),
        );
        biomes.insert(
            Hills,
            BiomeGeneration::new(None, 1.0, {
                let mut map = HashMap::new();
                map.insert(BiomeParameters::Temperature, -0.2);
                map.insert(BiomeParameters::Humidity, -0.2);
                map
            }),
        );
        biomes.insert(
            MagicForest,
            BiomeGeneration::new(None, 0.5, {
                let mut map = HashMap::new();
                map.insert(BiomeParameters::Temperature, 0.4);
                map.insert(BiomeParameters::Humidity, 0.3);
                map
            }),
        );
        biomes
    }
    pub fn default_generation_choices() -> HashMap<Biome, Vec<(Option<ItemType>, usize)>> {
        let mut generation_choices = HashMap::new();
        generation_choices.insert(Biome::Lake, vec![(None, 1)]);
        generation_choices.insert(
            Biome::Beach,
            vec![(None, 200), (Some(ItemType::TreasureMark), 1)],
        );
        generation_choices.insert(
            Biome::Forest,
            vec![
                (None, 300),
                (Some(ItemType::Tree), 30),
                (Some(ItemType::Stick), 10),
            ],
        );
        generation_choices.insert(
            Biome::Hills,
            vec![
                (None, 300),
                ((Some(ItemType::Pebble)), 20),
                ((Some(ItemType::Rock)), 10),
                ((Some(ItemType::GoldRock)), 1),
            ],
        );
        generation_choices.insert(
            Biome::MagicForest,
            vec![
                (None, 300),
                ((Some(ItemType::BigMushroom)), 10),
                ((Some(ItemType::MagicCrystal)), 1),
            ],
        );
        generation_choices
    }
    pub fn default_scores_map() -> HashMap<ItemType, i32> {
        let mut scores_map = HashMap::new();
        scores_map.insert(ItemType::TreasureChest, 5);
        scores_map.insert(ItemType::CrystalShard, 10);
        scores_map.insert(ItemType::GoldNugget, 5);
        scores_map.insert(ItemType::Stick, -1);
        scores_map.insert(ItemType::Pebble, -1);
        scores_map
    }
}
