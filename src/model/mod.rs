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
    world_name: String,
    rules: Rules,
    resource_pack: ResourcePack,
    seed: u32,
    generation_noises: HashMap<GenerationParameter, GenerationNoise>,
    chunk_size: Vec2<usize>,
    loaded_chunks: HashMap<Vec2<i64>, Chunk>,
    players: HashMap<Id, Player>,
    items: HashMap<Id, Item>,
    current_time: usize,
    sounds: HashMap<Id, Vec<Sound>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    RequestUpdate { load_area: Option<AABB<f32>> },
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
    pub fn create(world_name: String) -> Result<Self, anyhow::Error> {
        std::fs::create_dir_all(format!("saves/{}/chunks", world_name))?;
        serde_json::to_writer(
            std::io::BufWriter::new(std::fs::File::create(format!(
                "saves/{}/config.json",
                world_name
            ))?),
            &Config::default(),
        )?;
        Self::load(world_name)
    }
    pub fn load(world_name: String) -> Result<Self, anyhow::Error> {
        let config: Config = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(format!("saves/{}/config.json", world_name))?,
        ))?;
        let (pack_list, resource_pack) = ResourcePack::load_resource_packs().unwrap();
        let rules = Rules {
            player_movement_speed: config.player_movement_speed,
            client_view_distance: config.view_distance,
            campfire_light: config.campfire_light,
            torch_light: config.torch_light,
            statue_light: config.statue_light,
            regeneration_percent: config.regeneration_percent,
            player_interaction_range: config.player_interaction_range,
            sound_distance: config.sound_distance,
            generation_distance: config.generation_distance,
            spawn_area: config.spawn_area,
        };
        let mut model = Self {
            world_name,
            pack_list,
            rules,
            seed: config.seed,
            generation_noises: Self::init_generation_noises(config.seed, &resource_pack),
            resource_pack,
            ticks_per_second: config.ticks_per_second,
            chunk_size: config.chunk_size,
            loaded_chunks: HashMap::new(),
            players: HashMap::new(),
            items: HashMap::new(),
            current_time: 0,
            sounds: HashMap::new(),
        };
        model.load_chunks_at(vec2(0, 0));
        Ok(model)
    }
    pub fn drop_player(&mut self, player_id: Id) {
        self.players.remove(&player_id);
        self.sounds.remove(&player_id);
    }
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        let player = self.players.get_mut(&player_id).unwrap();
        match message {
            Message::RequestUpdate { load_area } => {
                if let Some(load_area) = load_area {
                    player.load_area = load_area;
                }
            }
            Message::Goto { pos } => {
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: None,
                });
            }
            Message::Interact { id } => {
                if let Some(item) = self.items.get(&id) {
                    player.action = Some(PlayerAction::MovingTo {
                        pos: item.pos,
                        finish_action: Some(MomentAction::Interact { id }),
                    });
                }
            }
            Message::Drop { pos } => {
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: Some(MomentAction::Drop { pos }),
                });
            }
            Message::PickUp { id } => {
                if let Some(item) = self.items.get(&id) {
                    player.action = Some(PlayerAction::MovingTo {
                        pos: item.pos,
                        finish_action: Some(MomentAction::PickUp { id }),
                    });
                }
            }
            Message::SayHi => {
                let pos = player.pos;
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
