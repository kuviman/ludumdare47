use super::*;

#[derive(ugli::Vertex)]
pub struct Vertex {
    pub a_pos: Vec3<f32>,
    pub a_color: Color<f32>,
}

#[derive(ugli::Vertex)]
pub struct Instance {
    pub i_pos: Vec3<f32>,
    pub i_size: f32,
}

pub struct Ez3D {
    geng: Rc<Geng>,
    program: ugli::Program,
}

impl Ez3D {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            program: geng
                .shader_lib()
                .compile(include_str!("program.glsl"))
                .unwrap(),
        }
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        vertices: &ugli::VertexBuffer<Vertex>,
        instances: impl Iterator<Item = Instance>,
    ) {
        let uniforms = camera.uniforms(framebuffer.size());
        ugli::draw(
            framebuffer,
            &self.program,
            ugli::DrawMode::Triangles,
            ugli::instanced(
                vertices,
                &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), instances.collect()),
            ),
            uniforms,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                depth_func: Some(default()),
                ..default()
            },
        );
    }
}
