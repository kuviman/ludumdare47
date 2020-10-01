use super::*;

mod controls;

pub use controls::*;

#[derive(Debug, Clone)]
pub struct Camera {
    pub center: Vec2<f32>,
    pub rotation: f32,
    pub attack: f32,
    pub distance: f32,
    pub perspective: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            center: vec2(0.0, 0.0),
            distance: 32.0,
            rotation: 0.0,
            attack: f32::PI / 2.0,
            perspective: true,
        }
    }
    fn view_matrix(&self) -> Mat4<f32> {
        Mat4::translate(vec3(0.0, 0.0, -self.distance))
            * Mat4::rotate_x(self.attack - f32::PI / 2.0)
            * Mat4::rotate_z(self.rotation)
            * Mat4::translate(-self.center.extend(0.0))
    }
    fn projection_matrix(&self, framebuffer_size: Vec2<usize>) -> Mat4<f32> {
        if self.perspective {
            Mat4::perspective(
                f32::PI / 2.0,
                framebuffer_size.x as f32 / framebuffer_size.y as f32,
                0.1,
                1e5,
            )
        } else {
            Mat4::scale(vec3(
                framebuffer_size.y as f32 / framebuffer_size.x as f32 / self.distance,
                1.0 / self.distance,
                -1e-3,
            ))
        }
    }
    pub fn uniforms(&self, framebuffer_size: Vec2<usize>) -> impl ugli::Uniforms {
        ugli::uniforms! {
            u_projection_matrix: self.projection_matrix(framebuffer_size),
            u_view_matrix: self.view_matrix(),
        }
    }
    pub fn screen_to_world(&self, framebuffer_size: Vec2<usize>, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = vec2(
            pos.x / framebuffer_size.x as f32 * 2.0 - 1.0,
            pos.y / framebuffer_size.y as f32 * 2.0 - 1.0,
        );
        // proj * view * (rx, ry, 0, 1 / w) = (px, py, ?, 1)
        let inv_matrix = (self.projection_matrix(framebuffer_size) * self.view_matrix()).inverse();
        let p1 = inv_matrix * pos.extend(0.0).extend(1.0);
        let p2 = inv_matrix * pos.extend(1.0).extend(1.0);
        let p1 = p1.xyz() / p1.w;
        let p2 = p2.xyz() / p2.w;
        let t = p1.z / (p1.z - p2.z);
        p1.xy() + (p2.xy() - p1.xy()) * t
    }
    pub fn world_to_screen(&self, framebuffer_size: Vec2<usize>, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = (self.projection_matrix(framebuffer_size) * self.view_matrix())
            * pos.extend(0.0).extend(1.0);
        let pos = pos.xy() / pos.w;
        vec2(
            (pos.x + 1.0) / 2.0 * framebuffer_size.x as f32,
            (pos.y + 1.0) / 2.0 * framebuffer_size.y as f32,
        )
    }
}
