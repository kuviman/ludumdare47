use super::*;

impl Model {
    pub fn new(config: Config) -> Self {
        let recipes = Config::default_recipes();
        let (tiles, height_map) = Self::generate_map(config.map_size);
        let rules = Rules {
            entity_day_view_distance: config.player_day_view_distance,
            entity_night_view_distance: config.player_night_view_distance,
            campfire_light: 5.0,
            torch_light: 5.0,
            fire_extinguish_chance: config.fire_extinguish_chance,
            regeneration_percent: 0.1,
        };
        let mut model = Self {
            rules,
            ticks_per_second: config.ticks_per_second,
            size: config.map_size,
            tiles,
            height_map,
            structures: HashMap::new(),
            entities: HashMap::new(),
            current_time: 0,
            day_length: config.day_length,
            night_length: config.night_length,
            recipes,
            generation_choices: Config::default_generation_choices(),
        };
        for y in 0..model.size.y {
            for x in 0..model.size.x {
                let pos = vec2(x, y);
                if model.is_empty_tile(pos) {
                    model.generate_tile(pos);
                }
            }
        }
        model
    }
    pub fn new_player(&mut self) -> Id {
        let player_id;
        if let Some(pos) = self.get_spawnable_pos(GroundType::Sand) {
            let entity = Entity {
                id: Id::new(),
                pos,
                size: vec2(1, 1),
                view_range: self.calc_view_range(),
                move_to: None,
                item: None,
                controllable: true,
                colors: EntityColors::new(),
            };
            player_id = entity.id;
            self.entities.insert(entity.id, entity);
        } else {
            error!("Did not find spawnable position");
            player_id = Id::new(); // TODO
        }
        player_id
    }
    fn generate_map(map_size: Vec2<usize>) -> (Vec<Vec<Tile>>, Vec<Vec<f32>>) {
        let noise = OpenSimplex::new().set_seed(global_rng().gen());
        let mut height_map = vec![];
        for y in 0..map_size.y + 1 {
            let mut row = vec![];
            for x in 0..map_size.x + 1 {
                let pos = vec2(x, y).map(|x| x as f32);
                let normalized_pos = vec2(pos.x / map_size.x as f32, pos.y / map_size.y as f32)
                    * 2.0
                    - vec2(1.0, 1.0);
                let height = 1.0 - normalized_pos.len() * 1.2
                    + (noise.get([normalized_pos.x as f64 * 5.0, normalized_pos.y as f64 * 5.0])
                        as f32
                        / 10.0);
                row.push(height);
            }
            height_map.push(row);
        }
        let mut tiles = vec![];
        for y in 0..map_size.y {
            let mut tiles_row = vec![];
            for x in 0..map_size.x {
                let water = height_map[x][y] < 0.0
                    || height_map[x + 1][y] < 0.0
                    || height_map[x + 1][y + 1] < 0.0
                    || height_map[x][y + 1] < 0.0;
                let middle_height = (height_map[x][y]
                    + height_map[x + 1][y]
                    + height_map[x + 1][y + 1]
                    + height_map[x][y + 1])
                    / 4.0;
                tiles_row.push(Tile {
                    pos: vec2(x, y),
                    ground_type: if water {
                        GroundType::Water
                    } else if middle_height < 0.05 {
                        GroundType::Sand
                    } else {
                        GroundType::Grass
                    },
                });
            }
            tiles.push(tiles_row);
        }
        (tiles, height_map)
    }
    pub fn generate_tile(&mut self, pos: Vec2<usize>) {
        let mut rng = global_rng();
        let choice = &self.generation_choices[&self.tiles[pos.y][pos.x].ground_type]
            .choose_weighted(&mut rng, |item| item.1)
            .unwrap()
            .0;
        if let Some(structure) = choice {
            let structure = Structure { pos, ..*structure };
            self.structures.insert(structure.pos, structure);
        }
    }
    fn is_spawnable_tile(&self, pos: Vec2<usize>) -> bool {
        self.tiles[pos.y][pos.x].ground_type != GroundType::Water && self.is_empty_tile(pos)
    }
    fn get_spawnable_pos(&self, ground_type: GroundType) -> Option<Vec2<usize>> {
        let mut positions = vec![];
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = vec2(x, y);
                if self.is_spawnable_tile(pos)
                    && self.get_tile(vec2(x, y)).unwrap().ground_type == ground_type
                {
                    positions.push(pos);
                }
            }
        }
        let length = positions.len();
        if length > 0 {
            positions.get(global_rng().gen_range(0, length)).copied()
        } else {
            None
        }
    }
}
