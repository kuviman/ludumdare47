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

impl Model {
    pub const TICKS_PER_SECOND: f32 = 1.0;
    pub fn new(config: Config) -> Self {
        let mut model = Self {
            size: config.map_size,
            tiles: Self::generate_tiles(config.map_size),
            structures: vec![],
        };
        model.gen_structures();
        model
    }
    pub fn tick(&mut self) {}
    pub fn new_player(&mut self) -> Id {
        Id::new()
    }
    pub fn drop_player(&mut self, player_id: Id) {}
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => println!("Got ping message"),
        }
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
            let x = global_rng().gen_range(0, self.size.x);
            let y = global_rng().gen_range(0, self.size.y);
            if GroundType::Water != self.tiles.get(y).unwrap().get(x).unwrap().ground_type
                && !self.structures.iter().any(|structure| {
                    x >= structure.pos.x
                        && x <= structure.pos.x + structure.size.x
                        && y >= structure.pos.y
                        && y <= structure.pos.y + structure.size.y
                })
            {
                let structure = Structure {
                    pos: vec2(x, y),
                    size: vec2(1, 1),
                    structure_type: StructureType::Tree,
                };
                self.structures.push(structure);
            }
        }
    }
}
