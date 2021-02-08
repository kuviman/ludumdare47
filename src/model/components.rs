use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompCollidable();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompPickable();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompPlayer {
    pub interaction_range: f32,
    #[serde(default)]
    pub item: Option<EntityType>,
    #[serde(default)]
    pub colors: PlayerColors,
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
