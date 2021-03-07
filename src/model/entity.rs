use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub entity_type: EntityType,
    pub id: Id,
    pub pos: Vec2<f32>,
    pub size: f32,
    pub components: EntityComponents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityComponents {
    #[serde(default)]
    pub player: Option<CompPlayer>,
    #[serde(default)]
    pub collidable: Option<CompCollidable>,
    #[serde(default)]
    pub pickable: Option<CompPickable>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct EntityType(pub String);

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityProperties {
    pub size: f32,
    pub components: EntityComponents,
}

impl Entity {
    pub fn new(
        entity_type: &EntityType,
        entity_properties: &EntityProperties,
        position: Vec2<f32>,
        id: Id,
    ) -> Self {
        Self {
            entity_type: entity_type.clone(),
            id,
            pos: position,
            size: entity_properties.size,
            components: entity_properties.components.clone(),
        }
    }
}
