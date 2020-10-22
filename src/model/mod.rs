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
    pub entities: HashMap<Id, Entity>,
    pub items: HashMap<Id, Item>,
    pub current_time: usize,
    pub day_length: usize,
    pub night_length: usize,
    pub recipes: Vec<Recipe>,
    pub scores_map: HashMap<ItemType, i32>,
    pub sound_distance: f32,
    generation_choices: HashMap<Biome, Vec<(Option<ItemType>, usize)>>,
    sounds: HashMap<Id, Vec<Sound>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Ping,
    Goto { pos: Vec2<usize> },
    Interact { id: Id },
    Drop,
    PickUp,
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
            Message::Ping => println!("Got ping message"),
            Message::Goto { pos } => {
                println!("Got Goto {}", pos);
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                if entity.controllable && pos.x < self.size.x && pos.y < self.size.y {
                    entity.move_to = Some((pos, false));
                }
            }
            Message::Interact { id } => {
                println!("Got Interact with {:?}", id);
                let mut entity = self.entities.get_mut(&player_id).unwrap();
                if let Some(item) = self.items.get(&id) {
                    if entity.controllable && item.pos.x < self.size.x && item.pos.y < self.size.y {
                        entity.move_to = Some((item.pos, true));
                    }
                }
            }
            Message::Drop => {
                println!("Got Drop");
                let mut entity = self.entities.get(&player_id).unwrap().clone();
                let hand_item = &mut entity.item;
                let mut item = self.remove_item(entity.pos);
                let ground_item = match &item {
                    Some(item) => Some(item.item_type),
                    None => None,
                };
                if let None = ground_item {
                    if let Some(item_type) = hand_item.take() {
                        self.spawn_item(item_type, entity.pos);
                        self.play_sound(Sound::PutDown, self.sound_distance, entity.pos);
                    }
                }
                if let Some(item) = item {
                    self.spawn_item(item.item_type, item.pos);
                }
                *self.entities.get_mut(&player_id).unwrap() = entity;
            }
            Message::PickUp => {
                println!("Got PickUp");
                let mut entity = self.entities.get(&player_id).unwrap().clone();
                let hand_item = &mut entity.item;
                let mut item = self.remove_item(entity.pos);
                let ground_item = match &item {
                    Some(item) => Some(item.item_type),
                    None => None,
                };
                if let None = hand_item {
                    if let Some(item_type) = ground_item {
                        if item_type.is_pickable() {
                            item.take();
                            *hand_item = Some(item_type);
                            self.play_sound(Sound::PickUp, self.sound_distance, entity.pos);
                        }
                    }
                }
                if let Some(item) = item {
                    self.spawn_item(item.item_type, item.pos);
                }
                *self.entities.get_mut(&player_id).unwrap() = entity;
            }
            Message::SayHi => {
                println!("Got SayHi");
                let entity = self.entities.get(&player_id).unwrap();
                self.play_sound(Sound::Hello, self.sound_distance, entity.pos);
            }
        }
    }
    fn get_tile(&self, pos: Vec2<usize>) -> Option<&Tile> {
        self.tiles.get(pos.y)?.get(pos.x)
    }
    fn is_empty_tile(&self, pos: Vec2<usize>) -> bool {
        self.items.values().find(|item| item.pos == pos).is_none()
            && !self.entities.values().any(|entity| pos == entity.pos)
    }
    fn is_traversable_tile(&self, pos: Vec2<usize>) -> bool {
        self.items
            .values()
            .find(|item| item.pos == pos)
            .map_or(true, |item| item.item_type.is_traversable())
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
