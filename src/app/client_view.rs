use super::*;
use client_entity::ClientEntity;

pub struct ClientView {
    pub players_online: usize,
    pub current_time: usize,
    pub ticks_per_second: f32,
    pub entities: Vec<ClientEntity>,
    pub recipes: Vec<model::Recipe>,
    pub sounds: Vec<model::Sound>,
}

impl ClientView {
    pub fn from_server_view(server_view: model::ClientView, resource_pack: &ResourcePack) -> Self {
        Self {
            players_online: server_view.players_online,
            current_time: server_view.current_time,
            ticks_per_second: server_view.ticks_per_second,
            recipes: server_view.recipes,
            sounds: server_view.sounds,
            entities: {
                server_view
                    .entities
                    .into_iter()
                    .map(|entity| ClientEntity::from_server_entity(entity, resource_pack))
                    .collect()
            },
        }
    }
    pub fn get_closest_entity(&self, pos: Vec2<f32>) -> Option<&ClientEntity> {
        self.entities
            .iter()
            .filter(|entity| entity.pos.is_some() && entity.size.is_some())
            .find(|entity| (entity.pos.unwrap() - pos).len() <= entity.size.unwrap())
    }
}
