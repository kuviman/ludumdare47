use super::*;

#[derive(Deref, DerefMut)]
pub struct ClientEntity {
    #[deref]
    #[deref_mut]
    pub server_entity: model::Entity,
    pub extra_components: ClientEntityComponents,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientEntityComponents {
    #[serde(default)]
    pub renderable: Option<CompRenderable>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum CompRenderable {
    Simple,
    Player,
}

impl ClientEntity {
    pub fn from_server_entity(server_entity: model::Entity, resource_pack: &ResourcePack) -> Self {
        Self {
            extra_components: {
                let mut components =
                    resource_pack.entity_components[&server_entity.entity_type].clone();
                components
            },
            server_entity,
        }
    }
}
