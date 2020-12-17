use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Rules {
    pub player_movement_speed: f32,
    pub client_view_distance: f32,
    pub campfire_light: f32,
    pub torch_light: f32,
    pub regeneration_percent: f32,
    pub statue_light: f32,
    pub player_interaction_range: f32,
    pub sound_distance: f32,
}
