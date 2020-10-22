use super::*;
use noise::{NoiseFn, OpenSimplex, Seedable};

mod config;
mod entity;
mod generation;
mod item;
mod pathfind;
mod player_view;
mod recipe;
mod rules;
mod tick;
mod tile;
mod vision;

pub use config::*;
pub use entity::*;
pub use generation::*;
pub use item::*;
pub use pathfind::*;
pub use player_view::*;
pub use recipe::*;
pub use rules::*;
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
    pub score: i32,
    pub ticks_per_second: f32,
    pub height_map: Vec<Vec<f32>>,
    pub size: Vec2<usize>,
    pub tiles: Vec<Vec<Tile>>,
    pub structures: HashMap<Vec2<usize>, Item>,
    pub entities: HashMap<Id, Entity>,
    pub items: HashMap<usize, Item>,
    pub current_time: usize,
    pub day_length: usize,
    pub night_length: usize,
    pub recipes: Vec<Recipe>,
    pub scores_map: HashMap<ItemType, i32>,
    pub sound_distance: f32,
    generation_choices: HashMap<Biome, Vec<(Option<Item>, usize)>>,
    sounds: HashMap<Id, Vec<Sound>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Ping,
    Click { pos: Vec2<usize>, secondary: bool },
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Trans)]
pub enum Sound {
    Craft,
    PickUp,
    PutDown,
    StatueGift,
    Hello,
}

impl Model {
    pub fn drop_player(&mut self, player_id: Id) {
        self.entities.remove(&player_id);
        self.sounds.remove(&player_id);
    }
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => println!("Got ping message"),
            Message::Click { pos, secondary } => {
                println!("Got click at {}:{}", pos, secondary);
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                if !secondary && pos == entity.pos {
                    self.play_sound(Sound::Hello, self.sound_distance, pos);
                } else if entity.controllable && pos.x < self.size.x && pos.y < self.size.y {
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
            .map_or(true, |structure| structure.item_type.is_traversable())
            && !self.entities.values().any(|entity| pos == entity.pos)
    }
    fn play_sound(&mut self, sound: Sound, range: f32, pos: Vec2<usize>) {
        for (id, entity_pos) in self.entities.iter().map(|(id, entity)| (id, entity.pos)) {
            let dx = pos.x as f32 - entity_pos.x as f32;
            let dy = pos.y as f32 - entity_pos.y as f32;
            if dx * dx + dy * dy <= range * range {
                self.sounds.get_mut(id).unwrap().push(sound);
            }
        }
    }
}
