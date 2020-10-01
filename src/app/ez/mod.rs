use super::*;

#[derive(ugli::Vertex)]
pub struct Vertex {
    pub a_pos: Vec2<f32>,
}

#[derive(ugli::Vertex)]
pub struct Instance {
    pub i_pos: Vec2<f32>,
    pub i_rotation: f32,
    pub i_size: Vec2<f32>,
    pub i_color: Color<f32>,
}

#[derive(ugli::Vertex, Clone)]
pub struct InstancedVertex {
    pub a_pos: Vec2<f32>,
    pub i_rotation: f32,
    pub i_pos: Vec2<f32>,
    pub i_size: Vec2<f32>,
    pub i_color: Color<f32>,
}

#[derive(Debug)]
pub struct Quad {
    pub pos: Vec2<f32>,
    pub rotation: f32,
    pub size: Vec2<f32>,
    pub color: Color<f32>,
}

impl From<Quad> for Instance {
    fn from(p: Quad) -> Self {
        Self {
            i_pos: p.pos,
            i_rotation: p.rotation,
            i_size: p.size,
            i_color: p.color,
        }
    }
}

pub struct Ez {
    geng: Rc<Geng>,
    program: ugli::Program,
    white_texture: ugli::Texture,
}

impl Ez {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            program: geng
                .shader_lib()
                .compile(include_str!("program.glsl"))
                .unwrap(),
            white_texture: {
                let mut texture =
                    ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Color::WHITE);
                texture.set_wrap_mode(ugli::WrapMode::Clamp);
                texture
            },
        }
    }
    fn ugli(&self) -> &Rc<Ugli> {
        self.geng.ugli()
    }
    fn internal_draw<V: ugli::VertexDataSource>(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        vertex_data: V,
        texture: Option<&ugli::Texture>,
        mode: ugli::DrawMode,
    ) {
        let texture = texture.unwrap_or(&self.white_texture);
        let uniforms = (
            camera.uniforms(framebuffer.size()),
            ugli::uniforms! {
                u_texture: texture,
            },
        );
        ugli::draw(
            framebuffer,
            &self.program,
            mode,
            vertex_data,
            uniforms,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        vertex_data: impl Iterator<Item = InstancedVertex>,
        texture: Option<&ugli::Texture>,
        mode: ugli::DrawMode,
    ) {
        let vertex_data = ugli::VertexBuffer::new_dynamic(self.ugli(), vertex_data.collect());
        self.internal_draw(framebuffer, camera, &vertex_data, texture, mode);
    }
    pub fn grid(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        size: Vec2<usize>,
        color: Color<f32>,
    ) {
        let mut points: Vec<Vec2<usize>> = Vec::new();
        for x in 0..=size.x {
            points.push(vec2(x, 0));
            points.push(vec2(x, size.y));
        }
        for y in 0..=size.y {
            points.push(vec2(0, y));
            points.push(vec2(size.x, y));
        }
        let vb = ugli::VertexBuffer::new_dynamic(
            self.ugli(),
            points
                .into_iter()
                .map(|p| InstancedVertex {
                    a_pos: p.map(|x| x as f32),
                    i_rotation: 0.0,
                    i_pos: vec2(0.0, 0.0),
                    i_size: vec2(1.0, 1.0),
                    i_color: color,
                })
                .collect(),
        );
        self.internal_draw(
            framebuffer,
            camera,
            &vb,
            None,
            ugli::DrawMode::Lines { line_width: 1.0 },
        );
    }
    fn quads_with<D>(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        data: D,
        mode: ugli::DrawMode,
        texture: Option<&ugli::Texture>,
    ) where
        D: IntoIterator<Item = Quad>,
    {
        let vb_v = ugli::VertexBuffer::new_dynamic(
            self.ugli(),
            vec![
                Vertex {
                    a_pos: vec2(-1.0, -1.0),
                },
                Vertex {
                    a_pos: vec2(1.0, -1.0),
                },
                Vertex {
                    a_pos: vec2(1.0, 1.0),
                },
                Vertex {
                    a_pos: vec2(-1.0, 1.0),
                },
            ],
        );
        let vb_i = ugli::VertexBuffer::new_dynamic(
            self.ugli(),
            data.into_iter()
                .map(|p| Instance {
                    i_pos: p.pos,
                    i_rotation: p.rotation,
                    i_size: p.size,
                    i_color: p.color,
                })
                .collect(),
        );
        self.internal_draw(
            framebuffer,
            camera,
            ugli::instanced(&vb_v, &vb_i),
            texture,
            mode,
        );
    }
    pub fn quads<D>(&self, framebuffer: &mut ugli::Framebuffer, camera: &Camera, data: D)
    where
        D: IntoIterator<Item = Quad>,
    {
        self.quads_with(framebuffer, camera, data, ugli::DrawMode::TriangleFan, None);
    }
    pub fn textured_quads<D>(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        data: D,
        texture: &ugli::Texture,
    ) where
        D: IntoIterator<Item = Quad>,
    {
        self.quads_with(
            framebuffer,
            camera,
            data,
            ugli::DrawMode::TriangleFan,
            Some(texture),
        );
    }
    pub fn frames<D>(&self, framebuffer: &mut ugli::Framebuffer, camera: &Camera, data: D)
    where
        D: IntoIterator<Item = Quad>,
    {
        self.quads_with(
            framebuffer,
            camera,
            data,
            ugli::DrawMode::LineLoop { line_width: 1.0 },
            None,
        );
    }
}
