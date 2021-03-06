use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rules {
    pub client_view_distance: f32,
    pub campfire_light: f32,
    pub torch_light: f32,
    pub regeneration_percent: f32,
    pub statue_light: f32,
    pub sound_distance: f32,
    pub generation_distance: usize,
    pub spawn_area: f32,
}
