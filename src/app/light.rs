use super::*;

#[derive(ugli::Uniforms)]
pub struct Uniforms {
    pub u_light_direction: Vec3<f32>,
}

impl Uniforms {
    pub fn new(model: &Model) -> Self {
        let is_day = model.current_time < model.day_length;
        if is_day {
            let angle = model.current_time as f32 / model.day_length as f32 * f32::PI;
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
