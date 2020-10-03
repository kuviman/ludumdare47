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
    pub player_view_distance: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map_size: vec2(20, 20),
            player_view_distance: 5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Model {
    pub entity_view_distance: usize,
    pub size: Vec2<usize>,
    pub tiles: Vec<Vec<Tile>>,
    pub structures: Vec<Structure>,
    pub entities: HashMap<Id, Entity>,
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
    pub height: f32,
    pub ground_type: GroundType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Structure {
    pub pos: Vec2<usize>,
    pub size: Vec2<usize>,
    pub structure_type: StructureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub enum StructureType {
    Tree,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Entity {
    pub pos: Vec2<usize>,
    pub size: Vec2<usize>,
    pub view_range: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct PlayerView {
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub structures: Vec<Structure>,
}

impl Model {
    pub const TICKS_PER_SECOND: f32 = 1.0;
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            entity_view_distance: config.player_view_distance,
            size: config.map_size,
            tiles: Self::generate_tiles(config.map_size),
            structures: vec![],
            entities: HashMap::new(),
        };
        model.gen_structures();
        model
    }
    pub fn tick(&mut self) {
        let ids: Vec<Id> = self.entities.keys().copied().collect();
        for id in ids {
            let mut entity = self.entities.get(&id).unwrap().clone();
            let dir = Self::get_random_dir();
            let new_pos = vec2(
                (entity.pos.x as i32 + dir.x) as usize,
                (entity.pos.y as i32 + dir.y) as usize,
            );
            if let Some(tile) = self.get_tile(new_pos) {
                if GroundType::Water != tile.ground_type && self.is_empty_tile(new_pos) {
                    entity.pos = new_pos;
                    *self.entities.get_mut(&id).unwrap() = entity;
                }
            }
        }
    }
    pub fn new_player(&mut self) -> Id {
        let id = Id::new();
        if let Some(pos) = self.get_spawnable_pos(100) {
            let entity = Entity {
                pos,
                size: vec2(1, 1),
                view_range: self.entity_view_distance,
            };
            self.entities.insert(id, entity);
        }
        id
    }
    pub fn drop_player(&mut self, player_id: Id) {}
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => println!("Got ping message"),
            Message::Click { pos, secondary } => println!("Got click at {}:{}", pos, secondary),
        }
    }
    pub fn get_view(&mut self, player_id: Id) -> PlayerView {
        let entity = self.entities.get(&player_id).unwrap();
        let mut view = vec![];
        view.push(entity.pos.clone());
        for x0 in 1..entity.view_range {
            view.push(vec2(x0, 0) + entity.pos);
            view.push(vec2(entity.pos.x - x0, entity.pos.y));
        }
        for y in 1..(entity.view_range + 1) {
            let x = ((entity.view_range * entity.view_range - y * y) as f32)
                .sqrt()
                .round() as usize;
            view.push(vec2(entity.pos.x, entity.pos.y + y));
            view.push(vec2(entity.pos.x, entity.pos.y - y));
            for x0 in 1..x {
                view.push(vec2(entity.pos.x + x0, entity.pos.y + y));
                view.push(vec2(entity.pos.x + x0, entity.pos.y - y));
                view.push(vec2(entity.pos.x - x0, entity.pos.y + y));
                view.push(vec2(entity.pos.x - x0, entity.pos.y - y));
            }
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
    fn get_tile(&self, pos: Vec2<usize>) -> Option<&Tile> {
        self.tiles.get(pos.y)?.get(pos.x)
    }
    fn is_empty_tile(&self, pos: Vec2<usize>) -> bool {
        !self.structures.iter().any(|structure| {
            pos.x >= structure.pos.x
                && pos.x <= structure.pos.x + structure.size.x - 1
                && pos.y >= structure.pos.y
                && pos.y <= structure.pos.y + structure.size.y - 1
        }) && !self.entities.values().any(|entity| {
            pos.x >= entity.pos.x
                && pos.x <= entity.pos.x + entity.size.x - 1
                && pos.y >= entity.pos.y
                && pos.y <= entity.pos.y + entity.size.y - 1
        })
    }
    fn is_under_view(&self, pos: Vec2<usize>) -> bool {
        self.entities.values().any(|entity| {
            let dx = pos.x - entity.pos.x;
            let dy = pos.y - entity.pos.y;
            let dist_sqr = dx * dx + dy * dy;
            dist_sqr <= entity.view_range * entity.view_range
        })
    }
    fn generate_tiles(map_size: Vec2<usize>) -> Vec<Vec<Tile>> {
        let noise = OpenSimplex::new().set_seed(global_rng().gen());
        let mut tiles = vec![];
        for y in 0..map_size.y {
            let mut tiles_row = vec![];
            for x in 0..map_size.x {
                let pos = vec2(x, y).map(|x| x as f32);
                let normalized_pos = vec2(pos.x / map_size.x as f32, pos.y / map_size.y as f32)
                    * 2.0
                    - vec2(1.0, 1.0);

                tiles_row.push(
                    if normalized_pos.len()
                        + (noise.get([normalized_pos.x as f64 * 5.0, normalized_pos.y as f64 * 5.0])
                            as f32
                            / 10.0)
                        > 0.8
                    {
                        Tile {
                            pos: vec2(x, y),
                            height: 0.0,
                            ground_type: GroundType::Water,
                        }
                    } else {
                        Tile {
                            pos: vec2(x, y),
                            height: 0.0,
                            ground_type: GroundType::Sand,
                        }
                    },
                );
            }
            tiles.push(tiles_row);
        }
        tiles
    }
    fn gen_structures(&mut self) {
        for _ in 0..10 {
            if let Some(pos) = self.get_spawnable_pos(1) {
                self.structures.push(Structure {
                    pos,
                    size: vec2(1, 1),
                    structure_type: StructureType::Tree,
                });
            }
        }
    }
    fn get_spawnable_pos(&self, max_attempts: usize) -> Option<Vec2<usize>> {
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
    fn get_random_dir() -> Vec2<i32> {
        let x = global_rng().gen_range(-1, 2);
        let y = global_rng().gen_range(-1, 2);
        vec2(x, y)
    }
}
