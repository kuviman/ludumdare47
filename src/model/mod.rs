use super::*;

mod chunk;
mod client_view;
mod config;
mod generation;
mod generation_noise;
mod item;
mod player;
mod recipe;
mod resource_pack;
mod rules;
mod tick;
mod tile;
mod vision;

pub use chunk::*;
pub use client_view::*;
pub use config::*;
pub use generation::*;
pub use generation_noise::*;
pub use item::*;
pub use player::*;
pub use recipe::*;
pub use resource_pack::*;
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

pub struct Model {
    pub ticks_per_second: f32,
    pub pack_list: Vec<String>,
    rules: Rules,
    resource_pack: ResourcePack,
    generation_noises: HashMap<GenerationParameter, GenerationNoise>,
    chunk_size: Vec2<usize>,
    chunks: HashMap<Vec2<i64>, Chunk>,
    players: HashMap<Id, Player>,
    items: HashMap<Id, Item>,
    current_time: usize,
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
        self.players.remove(&player_id);
        self.sounds.remove(&player_id);
    }
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Ping => {}
            Message::Goto { pos } => {
                let mut player = self.players.get_mut(&player_id).unwrap();
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: None,
                });
            }
            Message::Interact { id } => {
                if let Some(item) = self.items.get(&id) {
                    let mut player = self.players.get_mut(&player_id).unwrap();
                    player.action = Some(PlayerAction::MovingTo {
                        pos: item.pos,
                        finish_action: Some(MomentAction::Interact { id }),
                    });
                }
            }
            Message::Drop { pos } => {
                let mut player = self.players.get_mut(&player_id).unwrap();
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: Some(MomentAction::Drop { pos }),
                });
            }
            Message::PickUp { id } => {
                let mut player = self.players.get_mut(&player_id).unwrap();
                if let Some(item) = self.items.get(&id) {
                    player.action = Some(PlayerAction::MovingTo {
                        pos: item.pos,
                        finish_action: Some(MomentAction::PickUp { id }),
                    });
                }
            }
            Message::SayHi => {
                let pos = self.players.get(&player_id).unwrap().pos;
                self.play_sound(Sound::Hello, pos);
            }
        }
    }
    fn is_empty_tile(&self, pos: Vec2<i64>) -> bool {
        !self
            .items
            .values()
            .any(|item| pos == item.pos.map(|x| x as i64))
            && !self
                .players
                .values()
                .any(|player| pos == player.pos.map(|x| x as i64))
    }
    fn play_sound(&mut self, sound: Sound, pos: Vec2<f32>) {
        let range = self.rules.sound_distance;
        for (id, player_pos) in self.players.iter().map(|(id, player)| (id, player.pos)) {
            let dx = pos.x - player_pos.x;
            let dy = pos.y - player_pos.y;
            if dx * dx + dy * dy <= range * range {
                self.sounds.get_mut(id).unwrap().push(sound);
            }
        }
    }
}
