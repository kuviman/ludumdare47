use super::*;

mod biome;
mod chunked_world;
mod client_view;
mod config;
mod id;
mod item;
mod multi_noise;
mod player;
mod recipe;
mod resource_pack;
mod rules;
mod spawn;
mod tick;
mod tile;
mod world_gen;

pub use biome::*;
pub use chunked_world::*;
pub use client_view::*;
pub use config::*;
use geng::prelude::fmt::Formatter;
pub use id::*;
pub use item::*;
pub use multi_noise::*;
pub use player::*;
pub use recipe::*;
pub use resource_pack::*;
pub use rules::*;
pub use tick::*;
pub use tile::*;
pub use world_gen::*;

pub struct Model {
    pub ticks_per_second: f32,
    pub pack_list: Vec<String>,
    id_generator: util::Saved<IdGenerator>,
    world_name: String,
    rules: Rules,
    resource_pack: ResourcePack,
    chunked_world: ChunkedWorld,
    players: HashMap<Id, Player>,
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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Sound {
    Craft,
    PickUp,
    PutDown,
    StatueGift,
    Hello,
}

#[derive(Debug, Clone)]
struct WorldError {
    world_name: String,
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A world with the name {} already exists.",
            self.world_name
        )
    }
}

impl std::error::Error for WorldError {}

impl Model {
    pub fn create(world_name: &str) -> Result<Self, anyhow::Error> {
        if std::path::Path::new(&format!("saves/{}", world_name)).exists() {
            return Err(anyhow::Error::from(WorldError {
                world_name: world_name.to_owned(),
            }));
        }
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
    pub fn load(world_name: &str) -> Result<Self, anyhow::Error> {
        let world_path = std::path::Path::new("saves").join(world_name);
        let config: Config = serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(world_path.join("config.json"))?,
        ))?;
        let (pack_list, resource_pack) = ResourcePack::load_all("packs").unwrap();
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
        let world_gen = WorldGen::new(config.seed, &resource_pack);
        let mut model = Self {
            id_generator: util::Saved::new(world_path.join("id_gen"), IdGenerator::new),
            world_name: world_name.to_owned(),
            pack_list,
            rules,
            resource_pack,
            ticks_per_second: config.ticks_per_second,
            chunked_world: ChunkedWorld::new(world_path, config.chunk_size, world_gen),
            players: HashMap::new(),
            current_time: 0,
            sounds: HashMap::new(),
        };
        Ok(model)
    }
    pub fn drop_player(&mut self, player_id: Id) {
        self.players.remove(&player_id);
        self.sounds.remove(&player_id);
        self.chunked_world
            .set_load_area_for(player_id, &mut self.id_generator, None);
    }
    pub fn handle_message(
        &mut self,
        player_id: Id,
        message: Message,
        sender: &mut dyn geng::net::Sender<ServerMessage>,
    ) {
        let player = self.players.get_mut(&player_id).unwrap();
        match message {
            Message::RequestUpdate { load_area } => {
                if let Some(load_area) = load_area {
                    player.load_area = load_area;
                    self.chunked_world.set_load_area_for(
                        player_id,
                        &mut self.id_generator,
                        Some(load_area),
                    );
                }
                // TODO: Diff?
                sender.send(ServerMessage::UpdateClientView(self.get_view(player_id)));
            }
            Message::Goto { pos } => {
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: None,
                });
            }
            Message::Interact { id } => {
                if let Some(item) = self.chunked_world.get_item(id) {
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
                if let Some(item) = self.chunked_world.get_item(id) {
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
