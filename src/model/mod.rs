use super::*;
use noise::{NoiseFn, OpenSimplex, Seedable};

mod config;
mod entity;
mod generation;
mod player_view;
mod recipe;
mod rules;
mod structure;
mod tick;
mod tile;
mod vision;

pub use config::*;
pub use entity::*;
pub use generation::*;
pub use player_view::*;
pub use recipe::*;
pub use rules::*;
pub use structure::*;
pub use tick::*;
pub use tile::*;
pub use vision::*;

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

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Model {
    pub rules: Rules,
    pub ticks_per_second: f32,
    pub height_map: Vec<Vec<f32>>,
    pub size: Vec2<usize>,
    pub tiles: Vec<Vec<Tile>>,
    pub structures: HashMap<Vec2<usize>, Structure>,
    pub entities: HashMap<Id, Entity>,
    pub current_time: usize,
    pub day_length: usize,
    pub night_length: usize,
    pub recipes: Vec<Recipe>,
    generation_choices: HashMap<Biome, Vec<(Option<Structure>, usize)>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Ping,
    Click { pos: Vec2<usize>, secondary: bool },
}

impl Model {
    pub fn drop_player(&mut self, player_id: Id) {
        self.entities.remove(&player_id);
    }
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => println!("Got ping message"),
            Message::Click { pos, secondary } => {
                println!("Got click at {}:{}", pos, secondary);
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                if entity.controllable && pos.x < self.size.x && pos.y < self.size.y {
                    entity.move_to = Some((pos, secondary));
                }
            }
        }
    }
    fn get_tile(&self, pos: Vec2<usize>) -> Option<&Tile> {
        self.tiles.get(pos.y)?.get(pos.x)
    }
    fn is_empty_tile(&self, pos: Vec2<usize>) -> bool {
        self.structures.get(&pos).is_none()
            && !self.entities.values().any(|entity| pos == entity.pos)
    }
    fn is_traversable_tile(&self, pos: Vec2<usize>) -> bool {
        self.structures
            .get(&pos)
            .map_or(true, |structure| structure.structure_type.traversable())
            && !self.entities.values().any(|entity| pos == entity.pos)
    }
}
