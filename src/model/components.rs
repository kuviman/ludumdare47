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
pub struct CompPlayer {
    pub interaction_range: f32,
    #[serde(default)]
    pub action: Option<PlayerAction>,
    #[serde(skip, default = "CompPlayer::default_load_area")]
    pub load_area: AABB<f32>,
}

impl CompPlayer {
    fn default_load_area() -> AABB<f32> {
        AABB::pos_size(vec2(0.0, 0.0), vec2(0.0, 0.0))
    }
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompHolding {
    #[serde(default)]
    pub entity: Option<EntityType>,
}
