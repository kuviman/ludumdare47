use super::*;
use noise::{NoiseFn, OpenSimplex, Seedable};

mod config;
mod entity;
mod generation;
mod item;
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
    pub pack_list: Vec<String>,
    pub rules: Rules,
    pub score: i32,
    pub ticks_per_second: f32,
    pub chunk_size: Vec2<usize>,
    pub chunks: HashMap<Vec2<i64>, Chunk>,
    pub entities: HashMap<Id, Entity>,
    pub items: HashMap<Id, Item>,
    pub current_time: usize,
    pub recipes: Vec<Recipe>,
    pub scores_map: HashMap<ItemType, i32>,
    pub sound_distance: f32,
    generation_choices: HashMap<Biome, Vec<(Option<ItemType>, usize)>>,
    sounds: HashMap<Id, Vec<Sound>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Ping,
    Goto { pos: Vec2<f32> },
    Interact { id: Id },
    Drop { pos: Vec2<f32> },
    PickUp { id: Id },
    SayHi,
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
            Message::Ping => {}
            Message::Goto { pos } => {
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                entity.action = Some(EntityAction::MovingTo {
                    pos,
                    finish_action: None,
                });
            }
            Message::Interact { id } => {
                if let Some(item) = self.items.get(&id) {
                    let mut entity = self.entities.get_mut(&player_id).unwrap();
                    entity.action = Some(EntityAction::MovingTo {
                        pos: item.pos,
                        finish_action: Some(MomentAction::Interact { id }),
                    });
                }
            }
            Message::Drop { pos } => {
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                entity.action = Some(EntityAction::MovingTo {
                    pos,
                    finish_action: Some(MomentAction::Drop { pos }),
                });
            }
            Message::PickUp { id } => {
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                if let Some(item) = self.items.get(&id) {
                    entity.action = Some(EntityAction::MovingTo {
                        pos: item.pos,
                        finish_action: Some(MomentAction::PickUp { id }),
                    });
                }
            }
            Message::SayHi => {
                let pos = self.entities.get(&player_id).unwrap().pos;
                self.play_sound(Sound::Hello, self.sound_distance, pos);
            }
        }
    }
    fn is_empty_tile(&self, pos: Vec2<i64>) -> bool {
        !self
            .items
            .values()
            .any(|item| pos == item.pos.map(|x| x as i64))
            && !self
                .entities
                .values()
                .any(|entity| pos == entity.pos.map(|x| x as i64))
    }
    fn play_sound(&mut self, sound: Sound, range: f32, pos: Vec2<f32>) {
        for (id, entity_pos) in self.entities.iter().map(|(id, entity)| (id, entity.pos)) {
            let dx = pos.x - entity_pos.x;
            let dy = pos.y - entity_pos.y;
            if dx * dx + dy * dy <= range * range {
                self.sounds.get_mut(id).unwrap().push(sound);
            }
        }
    }
}
