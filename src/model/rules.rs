use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Rules {
    pub entity_movement_speed: f32,
    pub entity_day_view_distance: f32,
    pub entity_night_view_distance: f32,
    pub campfire_light: f32,
    pub torch_light: f32,
    pub fire_extinguish_chance: f32,
    pub regeneration_percent: f32,
    pub statue_light: f32,
    pub entity_interaction_range: f32,
}
