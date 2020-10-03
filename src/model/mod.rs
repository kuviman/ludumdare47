use super::*;
use noise::{NoiseFn, OpenSimplex, Seedable};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Copy, Trans)]
pub struct Id(usize);

impl Id {
    pub fn new() -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        Self(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
    pub fn raw(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub map_size: Vec2<usize>,
    pub player_day_view_distance: f32,
    pub player_night_view_distance: f32,
    pub day_length: usize,
    pub night_length: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map_size: vec2(20, 20),
            player_day_view_distance: 10.0,
            player_night_view_distance: 3.0,
            day_length: 100,
            night_length: 50,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Model {
    pub height_map: Vec<Vec<f32>>,
    pub entity_day_view_distance: f32,
    pub entity_night_view_distance: f32,
    pub size: Vec2<usize>,
    pub tiles: Vec<Vec<Tile>>,
    pub structures: Vec<Structure>,
    pub entities: HashMap<Id, Entity>,
    pub recipes: Vec<Recipe>,
    pub current_time: usize,
    pub day_length: usize,
    pub night_length: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Ping,
    Click { pos: Vec2<usize>, secondary: bool },
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq)]
pub enum GroundType {
    Water,
    Sand,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
    pub pos: Vec2<usize>,
    pub ground_type: GroundType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Structure {
    pub pos: Vec2<usize>,
    pub size: Vec2<usize>,
    pub traversable: bool,
    pub structure_type: StructureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Copy)]
pub enum StructureType {
    Item { item: Item },
    Tree,
    Campfire,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq, Copy)]
pub enum Item {
    Pebble,
    Stick,
    Axe,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Entity {
    pub pos: Vec2<usize>,
    pub size: Vec2<usize>,
    pub view_range: f32,
    pub move_to: Option<(Vec2<usize>, bool)>,
    pub item: Option<Item>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct PlayerView {
    pub height_map: Vec<Vec<f32>>,
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub structures: Vec<Structure>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Recipe {
    pub ingredient1: Option<Item>,
    pub ingredient2: Option<StructureType>,
    pub result1: Option<Item>,
    pub result2: Option<StructureType>,
}

impl Recipe {
    fn ingredients_equal(
        &self,
        ingredient1: Option<Item>,
        ingredient2: Option<StructureType>,
    ) -> bool {
        ingredient1 == self.ingredient1 && ingredient2 == self.ingredient2
    }
}

impl Model {
    pub const TICKS_PER_SECOND: f32 = 1.0;
    pub fn new(config: Config) -> Self {
        let recipe1 = Recipe {
            ingredient1: Some(Item::Stick),
            ingredient2: Some(StructureType::Item { item: Item::Pebble }),
            result1: Some(Item::Axe),
            result2: None,
        };
        let recipe2 = Recipe {
            ingredient1: Some(Item::Pebble),
            ingredient2: Some(StructureType::Item { item: Item::Stick }),
            result1: Some(Item::Axe),
            result2: None,
        };
        let recipe3 = Recipe {
            ingredient1: Some(Item::Axe),
            ingredient2: Some(StructureType::Tree),
            result1: Some(Item::Axe),
            result2: Some(StructureType::Item { item: Item::Stick }),
        };
        let recipe4 = Recipe {
            ingredient1: Some(Item::Stick),
            ingredient2: Some(StructureType::Item { item: Item::Stick }),
            result1: None,
            result2: Some(StructureType::Campfire),
        };
        let (tiles, height_map) = Self::generate_map(config.map_size);
        let mut model = Self {
            entity_day_view_distance: config.player_day_view_distance,
            entity_night_view_distance: config.player_night_view_distance,
            size: config.map_size,
            tiles,
            height_map,
            structures: vec![],
            entities: HashMap::new(),
            recipes: vec![recipe1, recipe2, recipe3, recipe4],
            current_time: 0,
            day_length: config.day_length,
            night_length: config.night_length,
        };
        model.gen_structures();
        model
    }
    pub fn tick(&mut self) {
        self.current_time += 1;
        if self.current_time >= self.day_length + self.night_length {
            self.current_time = 0;
        }
        let ids: Vec<Id> = self.entities.keys().copied().collect();
        for id in ids {
            let mut entity = self.entities.get(&id).unwrap().clone();
            let time = self.current_time as f32;
            let day = self.day_length as f32;
            let night = self.night_length as f32;
            let mut t = 2.0 * (time - day - night / 2.0).abs() / (day + night);
            if t > 1.0 {
                t = 2.0 - t;
            }
            entity.view_range = self.entity_night_view_distance
                + t * (self.entity_day_view_distance - self.entity_night_view_distance) as f32;
            if let Some(move_to) = entity.move_to {
                if move_to.1 {
                    if (entity.pos.x as i32 - move_to.0.x as i32).abs() <= 1
                        && (entity.pos.y as i32 - move_to.0.y as i32).abs() <= 1
                    {
                        let ingredient1 = &mut entity.item;
                        let structure = self.get_structure(move_to.0);
                        let ingredient2 = match structure {
                            Some((_, structure)) => Some(structure.structure_type),
                            None => None,
                        };
                        let recipe = self
                            .recipes
                            .iter()
                            .find(|recipe| recipe.ingredients_equal(*ingredient1, ingredient2));
                        if let Some(recipe) = recipe {
                            *ingredient1 = recipe.result1;
                            if let Some(structure_type) = recipe.result2 {
                                if let Some((structure_index, _)) = structure {
                                    self.structures
                                        .get_mut(structure_index)
                                        .unwrap()
                                        .structure_type = structure_type;
                                } else {
                                    self.structures.push(Structure {
                                        pos: move_to.0,
                                        size: vec2(1, 1),
                                        traversable: if let StructureType::Item { item: _ } =
                                            structure_type
                                        {
                                            true
                                        } else {
                                            false
                                        },
                                        structure_type: structure_type,
                                    })
                                }
                            } else if let Some((structure_index, _)) = structure {
                                self.structures.remove(structure_index);
                            }
                        } else if let Some(_) = ingredient1 {
                            if let None = ingredient2 {
                                self.structures.push(Structure {
                                    pos: move_to.0,
                                    size: vec2(1, 1),
                                    traversable: true,
                                    structure_type: StructureType::Item {
                                        item: ingredient1.take().unwrap(),
                                    },
                                })
                            }
                        } else {
                            if let Some(structure_type) = ingredient2 {
                                if let StructureType::Item { item } = structure_type {
                                    let index = structure.unwrap().0;
                                    self.structures.remove(index);
                                    *ingredient1 = Some(item);
                                }
                            }
                        }
                        entity.move_to = None;
                        *self.entities.get_mut(&id).unwrap() = entity;
                        continue;
                    }
                } else {
                    if entity.pos == move_to.0 {
                        entity.move_to = None;
                        *self.entities.get_mut(&id).unwrap() = entity;
                        continue;
                    }
                }
                let dir_x = (move_to.0.x as i32 - entity.pos.x as i32).signum();
                let dir_y = (move_to.0.y as i32 - entity.pos.y as i32).signum();
                let new_pos = vec2(
                    (entity.pos.x as i32 + dir_x) as usize,
                    (entity.pos.y as i32 + dir_y) as usize,
                );
                if let Some(tile) = self.get_tile(new_pos) {
                    if GroundType::Water != tile.ground_type && self.is_traversable_tile(new_pos) {
                        entity.pos = new_pos;
                    }
                }
            }
            *self.entities.get_mut(&id).unwrap() = entity;
        }
    }
    pub fn new_player(&mut self) -> Id {
        let id = Id::new();
        if let Some(pos) = self.get_spawnable_pos() {
            let entity = Entity {
                pos,
                size: vec2(1, 1),
                view_range: self.entity_day_view_distance,
                move_to: None,
                item: None,
            };
            self.entities.insert(id, entity);
        }
        id
    }
    pub fn drop_player(&mut self, player_id: Id) {}
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => println!("Got ping message"),
            Message::Click { pos, secondary } => {
                println!("Got click at {}:{}", pos, secondary);
                if pos.x < self.size.x && pos.y < self.size.y {
                    let mut entity = self.entities.get_mut(&player_id).unwrap();
                    entity.move_to = Some((pos, secondary));
                }
            }
        }
    }
    pub fn get_view(&self, player_id: Id) -> PlayerView {
        let entity = self.entities.get(&player_id).unwrap();
        let mut view = vec![];
        Self::add_view_radius(&mut view, entity.pos, entity.view_range);
        for light_source in self
            .structures
            .iter()
            .filter(|structure| structure.structure_type == StructureType::Campfire)
        {
            Self::add_view_radius(&mut view, light_source.pos, 5.0);
        }

        let vision = PlayerView {
            tiles: {
                let mut tiles = vec![];
                for y in 0..self.size.y {
                    let tile_row = self.tiles.get(y).unwrap();
                    for x in 0..self.size.x {
                        let pos = vec2(x, y);
                        if view.contains(&pos) {
                            tiles.push(tile_row.get(x).unwrap().clone());
                        }
                    }
                }
                tiles
            },
            height_map: self.height_map.clone(),
            entities: self
                .entities
                .iter()
                .filter(|(_, entity)| view.contains(&entity.pos))
                .map(|(_, entity)| entity.clone())
                .collect(),
            structures: self
                .structures
                .iter()
                .filter(|structure| view.contains(&structure.pos))
                .map(|structure| structure.clone())
                .collect(),
        };
        vision
    }
    fn add_view_radius(view: &mut Vec<Vec2<usize>>, center_pos: Vec2<usize>, radius: f32) {
        view.push(center_pos.clone());
        for x0 in 1..(radius.round() as usize) {
            view.push(vec2(x0, 0) + center_pos);
            view.push(vec2(center_pos.x - x0, center_pos.y));
        }
        for y in 1..(radius.round() as usize + 1) {
            let x = (radius * radius - y as f32 * y as f32).sqrt().round() as usize;
            view.push(vec2(center_pos.x, center_pos.y + y));
            view.push(vec2(center_pos.x, center_pos.y - y));
            for x0 in 1..x {
                view.push(vec2(center_pos.x + x0, center_pos.y + y));
                view.push(vec2(center_pos.x + x0, center_pos.y - y));
                view.push(vec2(center_pos.x - x0, center_pos.y + y));
                view.push(vec2(center_pos.x - x0, center_pos.y - y));
            }
        }
    }
    fn get_tile(&self, pos: Vec2<usize>) -> Option<&Tile> {
        self.tiles.get(pos.y)?.get(pos.x)
    }
    fn get_structure(&self, pos: Vec2<usize>) -> Option<(usize, &Structure)> {
        self.structures
            .iter()
            .enumerate()
            .find(|(_, structure)| Self::is_pos_inside(pos, structure.pos, structure.size))
    }
    fn is_empty_tile(&self, pos: Vec2<usize>) -> bool {
        !self
            .structures
            .iter()
            .any(|structure| Self::is_pos_inside(pos, structure.pos, structure.size))
            && !self
                .entities
                .values()
                .any(|entity| Self::is_pos_inside(pos, entity.pos, entity.size))
    }
    fn is_traversable_tile(&self, pos: Vec2<usize>) -> bool {
        !self
            .structures
            .iter()
            .filter(|structure| !structure.traversable)
            .any(|structure| Self::is_pos_inside(pos, structure.pos, structure.size))
            && !self
                .entities
                .values()
                .any(|entity| Self::is_pos_inside(pos, entity.pos, entity.size))
    }
    fn is_pos_inside(
        pos: Vec2<usize>,
        structure_pos: Vec2<usize>,
        structure_size: Vec2<usize>,
    ) -> bool {
        pos.x >= structure_pos.x
            && pos.x <= structure_pos.x + structure_size.x - 1
            && pos.y >= structure_pos.y
            && pos.y <= structure_pos.y + structure_size.y - 1
    }
    fn is_under_view(&self, pos: Vec2<usize>) -> bool {
        self.entities.values().any(|entity| {
            let dx = pos.x - entity.pos.x;
            let dy = pos.y - entity.pos.y;
            let dist_sqr = (dx * dx + dy * dy) as f32;
            dist_sqr <= entity.view_range * entity.view_range
        })
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
                let height = 0.8 - normalized_pos.len()
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
                tiles_row.push(Tile {
                    pos: vec2(x, y),
                    ground_type: if water {
                        GroundType::Water
                    } else {
                        GroundType::Sand
                    },
                });
            }
            tiles.push(tiles_row);
        }
        (tiles, height_map)
    }
    fn gen_structures(&mut self) {
        self.spawn_structure(10, |pos| Structure {
            pos,
            size: vec2(1, 1),
            traversable: false,
            structure_type: StructureType::Tree,
        });
        self.spawn_structure(self.size.x / 2, |pos| Structure {
            pos,
            size: vec2(1, 1),
            traversable: true,
            structure_type: StructureType::Item { item: Item::Pebble },
        });
        self.spawn_structure(self.size.x / 2, |pos| Structure {
            pos,
            size: vec2(1, 1),
            traversable: true,
            structure_type: StructureType::Item { item: Item::Stick },
        });
    }
    fn spawn_structure(&mut self, count: usize, structure: fn(Vec2<usize>) -> Structure) {
        for _ in 0..count {
            if let Some(pos) = self.get_spawnable_pos() {
                self.structures.push(structure(pos));
            }
        }
    }
    fn get_spawnable_pos(&self) -> Option<Vec2<usize>> {
        let mut positions = vec![];
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = vec2(x, y);
                if GroundType::Water != self.tiles.get(y).unwrap().get(x).unwrap().ground_type
                    && self.is_empty_tile(pos)
                    && !self.is_under_view(pos)
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