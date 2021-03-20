use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Deref, DerefMut)]
pub struct Entity {
    pub entity_type: EntityType,
    pub id: Id,
    #[deref]
    #[deref_mut]
    pub components: EntityComponents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityComponents {
    #[serde(default)]
    pub renderable: Option<CompRenderable>,
    #[serde(default)]
    pub pos: Option<Vec2<f32>>,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub controller: Option<CompController>,
    #[serde(default)]
    pub player: Option<CompPlayer>,
    #[serde(default)]
    pub collidable: Option<CompCollidable>,
    #[serde(default)]
    pub pickable: Option<CompPickable>,
    #[serde(default)]
    pub holding: Option<CompHolding>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct EntityType(pub String);

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Entity {
    pub fn new(entity_type: &EntityType, components: EntityComponents, id: Id) -> Self {
        Self {
            entity_type: entity_type.clone(),
            id,
            components,
        }
    }
}
