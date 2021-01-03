use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub seed: u32,
    pub ticks_per_second: f32,
    pub chunk_size: Vec2<usize>,
    pub player_movement_speed: f32,
    pub view_distance: f32,
    pub regeneration_percent: f32,
    pub campfire_light: f32,
    pub torch_light: f32,
    pub statue_light: f32,
    pub sound_distance: f32,
    pub player_interaction_range: f32,
    pub generation_distance: usize,
    pub spawn_area: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 0,
            ticks_per_second: 20.0,
            chunk_size: vec2(10, 10),
            player_movement_speed: 2.0,
            view_distance: 20.0,
            regeneration_percent: 0.01,
            campfire_light: 5.0,
            torch_light: 5.0,
            statue_light: 10.0,
            sound_distance: 5.0,
            player_interaction_range: 1.5,
            generation_distance: 5,
            spawn_area: 5,
        }
    }
}
