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
pub enum CompController {
    Player {
        #[serde(default = "PlayerColors::new")]
        colors: PlayerColors,
    },
    BiomeRandomWalker {
        biome: Biome,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompHolding {
    #[serde(default)]
    pub entity: Option<Box<Entity>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompHP {
    pub max_hp: f32,
    pub current_hp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompWeapon {
    pub attack_time: f32,
    pub damage: f32,
    pub attack_distance: f32,
}
