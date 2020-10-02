use super::*;
use noise::{NoiseFn, OpenSimplex};

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map_size: vec2(20, 20),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Model {
    pub size: Vec2<usize>,
    pub tiles: Vec<Vec<Tile>>,
    pub structures: Vec<Structure>,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Ping,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans, PartialEq, Eq)]
pub enum GroundType {
    Water,
    Sand,
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Tile {
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
}

impl Model {
    pub const TICKS_PER_SECOND: f32 = 1.0;
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            size: config.map_size,
            tiles: Self::generate_tiles(config.map_size),
            structures: vec![],
            entities: vec![],
        };
        model.gen_structures();
        model
    }
    pub fn tick(&mut self) {
        for i in 0..self.entities.len() {
            let mut entity = self.entities[i].clone();
            let dir = Self::get_random_dir();
            let new_pos = vec2(
                (entity.pos.x as i32 + dir.x) as usize,
                (entity.pos.y as i32 + dir.y) as usize,
            );
            if let Some(tile) = self.get_tile(new_pos) {
                if GroundType::Water != tile.ground_type && self.is_empty_tile(new_pos) {
                    entity.pos = new_pos;
                    self.entities[i] = entity;
                }
            }
        }
    }
    pub fn new_player(&mut self) -> Id {
        if let Some(pos) = self.get_spawnable_pos(100) {
            let entity = Entity {
                pos,
                size: vec2(1, 1),
            };
            self.entities.push(entity);
        }

        Id::new()
    }
    pub fn drop_player(&mut self, player_id: Id) {}
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => println!("Got ping message"),
        }
    }
    fn get_tile(&self, pos: Vec2<usize>) -> Option<&Tile> {
        self.tiles.get(pos.y)?.get(pos.x)
    }
    fn is_empty_tile(&self, pos: Vec2<usize>) -> bool {
        !self.structures.iter().any(|structure| {
            pos.x >= structure.pos.x
                && pos.x <= structure.pos.x + structure.size.x
                && pos.y >= structure.pos.y
                && pos.y <= structure.pos.y + structure.size.y
        }) && !self.entities.iter().any(|entity| {
            pos.x >= entity.pos.x
                && pos.x <= entity.pos.x + entity.size.x
                && pos.y >= entity.pos.y
                && pos.y <= entity.pos.y + entity.size.y
        })
    }
    fn generate_tiles(map_size: Vec2<usize>) -> Vec<Vec<Tile>> {
        let noise = OpenSimplex::new();
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
                            height: 0.0,
                            ground_type: GroundType::Water,
                        }
                    } else {
                        Tile {
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
        for _ in 0..max_attempts {
            let x = global_rng().gen_range(0, self.size.x);
            let y = global_rng().gen_range(0, self.size.y);
            let pos = vec2(x, y);
            if GroundType::Water != self.tiles.get(y).unwrap().get(x).unwrap().ground_type
                && self.is_empty_tile(pos)
            {
                return Some(pos);
            }
        }
        None
    }
    fn get_random_dir() -> Vec2<i32> {
        let x = global_rng().gen_range(-1, 2);
        let y = global_rng().gen_range(-1, 2);
        vec2(x, y)
    }
}
