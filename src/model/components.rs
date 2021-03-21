use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompCollidable {
    pub collision_type: CollisionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionType {
    Static,
    Pushable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompPickable();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompRenderable {
    Simple,
    Player {
        #[serde(default)]
        colors: PlayerColors,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompController {
    PlayerController,
    MobController,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompHolding {
    #[serde(default)]
    pub entity: Option<EntityType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompAction {
    #[serde(default)]
    pub current_action: Option<EntityAction>,
    #[serde(default)]
    pub next_action: Option<EntityAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompInteraction {
    pub interaction_range: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompLoadArea {
    #[serde(skip, default = "CompLoadArea::default_load_area")]
    pub load_area: AABB<f32>,
}

impl CompLoadArea {
    fn default_load_area() -> AABB<f32> {
        AABB::pos_size(vec2(0.0, 0.0), vec2(0.0, 0.0))
    }
}
