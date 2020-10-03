use super::*;

mod obj;

pub use obj::*;

#[derive(ugli::Vertex, Debug, Clone)]
pub struct Vertex {
    pub a_pos: Vec3<f32>,
    pub a_normal: Vec3<f32>,
    pub a_color: Color<f32>,
}

pub fn calc_normals(vs: &mut [Vertex]) {
    for face in vs.chunks_exact_mut(3) {
        let n =
            Vec3::cross(face[1].a_pos - face[0].a_pos, face[2].a_pos - face[0].a_pos).normalize();
        for v in face {
            v.a_normal = n;
        }
    }
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
                cull_face: Some(ugli::CullFace::Back),
                ..default()
            },
        );
    }
}
