use super::*;

#[derive(ugli::Uniforms)]
pub struct Uniforms {
    pub u_light_direction: Vec3<f32>,
}

impl Uniforms {
    pub fn new(view: &model::PlayerView) -> Self {
        let is_day = view.current_time < view.day_length;
        if is_day {
            let angle = view.current_time as f32 / view.day_length as f32 * f32::PI;
            Self {
                u_light_direction: vec3(angle.cos(), 0.0, angle.sin()) * angle.sin(),
            }
        } else {
            Self {
                u_light_direction: vec3(0.0, 0.0, 0.0),
            }
        }
    }
}
