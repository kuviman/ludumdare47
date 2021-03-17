use super::*;

mod biome;
mod chunked_world;
mod client_view;
mod components;
mod config;
mod entity;
mod id;
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
pub use components::*;
pub use config::*;
pub use entity::*;
use geng::prelude::fmt::Formatter;
pub use id::*;
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
struct WorldExistsError {
    world_name: String,
}

impl std::fmt::Display for WorldExistsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A world with the name {} already exists.",
            self.world_name
        )
    }
}

impl std::error::Error for WorldExistsError {}

#[derive(Debug, Clone)]
struct WorldPackConflictError {}

impl std::fmt::Display for WorldPackConflictError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Loaded resource packs do not match loaded world's packs."
        )
    }
}

impl std::error::Error for WorldPackConflictError {}

impl Model {
    pub fn create(world_name: &str) -> Result<Self, anyhow::Error> {
        fn save_to<T: Serialize>(
            path: impl AsRef<std::path::Path>,
            value: &T,
        ) -> Result<(), std::io::Error> {
            Ok(serde_json::to_writer(
                std::io::BufWriter::new(std::fs::File::create(path)?),
                value,
            )?)
        }

        let world_path = std::path::Path::new("saves").join(world_name);
        if world_path.exists() {
            return Err(anyhow::Error::from(WorldExistsError {
                world_name: world_name.to_owned(),
            }));
        }
        let (pack_list, resource_pack) = model::ResourcePack::load_all("packs")?;
        std::fs::create_dir_all(world_path.join("chunks"))?;
        save_to(world_path.join("config.json"), &Config::default())?;
        save_to(world_path.join("pack_list"), &pack_list)?;
        Ok(Self::new(
            world_name,
            Config::default(),
            pack_list,
            resource_pack,
        ))
    }
    pub fn load(world_name: &str) -> Result<Self, anyhow::Error> {
        fn load_from<T: for<'de> Deserialize<'de>>(
            path: impl AsRef<std::path::Path>,
        ) -> Result<T, std::io::Error> {
            Ok(serde_json::from_reader(std::io::BufReader::new(
                std::fs::File::open(path)?,
            ))?)
        }

        let (pack_list, resource_pack) = model::ResourcePack::load_all("packs")?;
        let world_path = std::path::Path::new("saves").join(world_name);
        let world_pack_list: Vec<String> = load_from(world_path.join("pack_list"))?;
        if world_pack_list != pack_list {
            return Err(anyhow::Error::from(WorldPackConflictError {}));
        }
        let config: Config = load_from(world_path.join("config.json"))?;
        Ok(Self::new(world_name, config, pack_list, resource_pack))
    }
    fn new(
        world_name: &str,
        config: Config,
        pack_list: Vec<String>,
        resource_pack: ResourcePack,
    ) -> Self {
        let world_path = std::path::Path::new("saves").join(world_name);
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
        Self {
            id_generator: util::Saved::new(world_path.join("id_gen"), IdGenerator::new),
            world_name: world_name.to_owned(),
            pack_list,
            rules,
            resource_pack,
            ticks_per_second: config.ticks_per_second,
            chunked_world: ChunkedWorld::new(world_path, config.chunk_size, world_gen),
            current_time: 0,
            sounds: HashMap::new(),
        }
    }
    pub fn drop_player(&mut self, player_id: Id) {
        self.chunked_world.remove_entity(player_id);
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
        let mut entity = self
            .chunked_world
            .get_entity_mut(player_id)
            .unwrap()
            .clone();
        let mut player = entity.components.player.as_mut().unwrap();
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
                self.chunked_world.get_updates(player_id, sender);
            }
            Message::Goto { pos } => {
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: None,
                });
            }
            Message::Interact { id } => {
                if let Some(item) = self.chunked_world.get_entity(id) {
                    if let Some(item_pos) = item.pos {
                        player.action = Some(PlayerAction::MovingTo {
                            pos: item_pos,
                            finish_action: Some(MomentAction::Interact { id }),
                        });
                    }
                }
            }
            Message::Drop { pos } => {
                player.action = Some(PlayerAction::MovingTo {
                    pos,
                    finish_action: Some(MomentAction::Drop { pos }),
                });
            }
            Message::PickUp { id } => {
                if let Some(item) = self.chunked_world.get_entity(id) {
                    if let Some(item_pos) = item.pos {
                        player.action = Some(PlayerAction::MovingTo {
                            pos: item_pos,
                            finish_action: Some(MomentAction::PickUp { id }),
                        });
                    }
                }
            }
            Message::SayHi => {
                if let Some(pos) = entity.pos {
                    self.play_sound(Sound::Hello, pos);
                }
            }
        }
        *self.chunked_world.get_entity_mut(player_id).unwrap() = entity;
    }
    fn play_sound(&mut self, sound: Sound, pos: Vec2<f32>) {
        let range = self.rules.sound_distance;
        for entity in self
            .chunked_world
            .find_range(pos, range, |e| e.components.player.is_some())
        {
            self.sounds.get_mut(&entity.id).unwrap().push(sound);
        }
    }
}

fn get_tile_pos(pos: Vec2<f32>) -> Vec2<i64> {
    pos.map(|x| x.floor() as i64)
}
