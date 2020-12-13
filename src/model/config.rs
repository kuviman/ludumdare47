use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ticks_per_second: f32,
    pub chunk_size: Vec2<usize>,
    pub initial_generation_size: Vec2<usize>,
    pub player_movement_speed: f32,
    pub player_day_view_distance: f32,
    pub player_night_view_distance: f32,
    pub day_length: usize,
    pub night_length: usize,
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
            chunk_size: vec2(10, 10),
            initial_generation_size: vec2(20, 20),
            player_movement_speed: 2.0,
            player_day_view_distance: 10.0,
            player_night_view_distance: 3.0,
            day_length: 1000,
            night_length: 500,
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
                result1: Some(SharpStone),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(Pebble),
                ingredient2: Some(Rock),
                result1: Some(SharpStone),
                result2: Some(Rock),
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(Pebble),
                result1: Some(Shovel),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(Pebble),
                ingredient2: Some(Stick),
                result1: Some(Shovel),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(SharpStone),
                result1: Some(Axe),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(SharpStone),
                ingredient2: Some(Stick),
                result1: Some(Axe),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(Axe),
                ingredient2: Some(SharpStone),
                result1: Some(Pickaxe),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(SharpStone),
                ingredient2: Some(Axe),
                result1: Some(Pickaxe),
                result2: None,
                conditions: None,
                craft_time: 0.1,
            },
            Recipe {
                ingredient1: Some(Pickaxe),
                ingredient2: Some(Rock),
                result1: Some(Pickaxe),
                result2: Some(SharpStone),
                conditions: None,
                craft_time: 1.0,
            },
            Recipe {
                ingredient1: Some(Pickaxe),
                ingredient2: Some(GoldRock),
                result1: Some(Pickaxe),
                result2: Some(GoldNugget),
                conditions: None,
                craft_time: 1.0,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(GoldNugget),
                result1: Some(GoldPickaxe),
                result2: None,
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(GoldNugget),
                ingredient2: Some(Stick),
                result1: Some(GoldPickaxe),
                result2: None,
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(Stick),
                result1: None,
                result2: Some(DoubleStick),
                conditions: None,
                craft_time: 0.01,
            },
            Recipe {
                ingredient1: Some(Axe),
                ingredient2: Some(Tree),
                result1: Some(Axe),
                result2: Some(Log),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Axe),
                ingredient2: Some(Log),
                result1: Some(Axe),
                result2: Some(Planks),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Log),
                ingredient2: Some(DoubleStick),
                result1: None,
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Log),
                ingredient2: Some(Planks),
                result1: None,
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Torch),
                ingredient2: Some(Log),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Torch),
                ingredient2: Some(Planks),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(Torch),
                ingredient2: Some(DoubleStick),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(DoubleStick),
                ingredient2: Some(Log),
                result1: None,
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.5,
            },
            Recipe {
                ingredient1: Some(DoubleStick),
                ingredient2: None,
                result1: Some(Stick),
                result2: Some(Stick),
                conditions: None,
                craft_time: 0.01,
            },
            Recipe {
                ingredient1: Some(Stick),
                ingredient2: Some(Campfire),
                result1: Some(Torch),
                result2: Some(Campfire),
                conditions: None,
                craft_time: 0.01,
            },
            Recipe {
                ingredient1: Some(GoldPickaxe),
                ingredient2: Some(MagicCrystal),
                result1: Some(GoldPickaxe),
                result2: Some(CrystalShard),
                conditions: None,
                craft_time: 1.0,
            },
            Recipe {
                ingredient1: Some(GoldPickaxe),
                ingredient2: Some(Rock),
                result1: Some(GoldPickaxe),
                result2: Some(SharpStone),
                conditions: None,
                craft_time: 1.0,
            },
            Recipe {
                ingredient1: Some(GoldPickaxe),
                ingredient2: Some(GoldRock),
                result1: Some(GoldPickaxe),
                result2: Some(GoldNugget),
                conditions: None,
                craft_time: 1.0,
            },
            Recipe {
                ingredient1: Some(Shovel),
                ingredient2: Some(TreasureMark),
                result1: Some(Shovel),
                result2: Some(TreasureChest),
                conditions: None,
                craft_time: 0.5,
            },
        ]
    }
    pub fn default_parameters() -> HashMap<BiomeParameter, NoiseParameters> {
        let mut map = HashMap::new();
        map.insert(
            BiomeParameter::Height,
            NoiseParameters::new(-1.0, 1.0, 100.0, 3, 2.0, 0.5),
        );
        map.insert(
            BiomeParameter::Magic,
            NoiseParameters::new(0.0, 1.0, 50.0, 1, 1.0, 1.0),
        );
        map.insert(
            BiomeParameter::Humidity,
            NoiseParameters::new(0.0, 1.0, 50.0, 1, 1.0, 1.0),
        );
        map
    }
    pub fn default_biomes() -> HashMap<Biome, BiomeGeneration> {
        use Biome::*;

        let mut biomes = HashMap::new();
        biomes.insert(
            Ocean,
            BiomeGeneration::new(0.0, {
                let mut map = HashMap::new();
                map.insert(BiomeParameter::Height, (-1.0, -0.3));
                map
            }),
        );
        biomes.insert(
            Beach,
            BiomeGeneration::new(0.2, {
                let mut map = HashMap::new();
                map.insert(BiomeParameter::Height, (-0.3, -0.2));
                map
            }),
        );
        biomes.insert(
            Forest,
            BiomeGeneration::new(0.0, {
                let mut map = HashMap::new();
                map.insert(BiomeParameter::Height, (-0.2, 0.6));
                map
            }),
        );
        biomes.insert(
            Lake,
            BiomeGeneration::new(0.1, {
                let mut map = HashMap::new();
                map.insert(BiomeParameter::Height, (0.2, 0.5));
                map.insert(BiomeParameter::Humidity, (0.9, 1.0));
                map
            }),
        );
        biomes.insert(
            MagicForest,
            BiomeGeneration::new(0.1, {
                let mut map = HashMap::new();
                map.insert(BiomeParameter::Height, (0.2, 0.5));
                map.insert(BiomeParameter::Magic, (0.8, 1.0));
                map
            }),
        );
        biomes.insert(
            Hills,
            BiomeGeneration::new(0.0, {
                let mut map = HashMap::new();
                map.insert(BiomeParameter::Height, (0.6, 1.0));
                map
            }),
        );
        biomes
    }
    pub fn default_generation_choices() -> HashMap<Biome, Vec<(Option<ItemType>, usize)>> {
        let mut generation_choices = HashMap::new();
        generation_choices.insert(Biome::Ocean, vec![(None, 1)]);
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
