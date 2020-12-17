use super::*;

#[derive(ugli::Uniforms)]
pub struct Uniforms {
    pub u_light_direction: Vec3<f32>,
}

impl Uniforms {
    pub fn new(_view: &model::ClientView) -> Self {
        let angle = f32::PI / 3.0;
        Self {
            u_light_direction: vec3(angle.cos(), 0.0, angle.sin()) * angle.sin(),
        }
    }
}
