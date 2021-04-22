use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Spawn { entity_type: EntityType },
    Heal { heal_points: f32 },
}
