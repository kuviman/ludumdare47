use super::*;

pub struct TileMesh {
    ez3d: Rc<Ez3D>,
    noise: noise::OpenSimplex,
    pub tiles: HashMap<Vec2<i64>, model::Tile>,
    pub mesh: ugli::VertexBuffer<ez3d::Vertex>,
    resource_pack: Rc<ResourcePack>,
}

impl TileMesh {
    pub fn new(geng: &Rc<Geng>, ez3d: &Rc<Ez3D>, resource_pack: &Rc<ResourcePack>) -> Self {
        Self {
            ez3d: ez3d.clone(),
            noise: noise::OpenSimplex::new(),
            tiles: HashMap::new(),
            mesh: ugli::VertexBuffer::new_static(geng.ugli(), Vec::new()),
            resource_pack: resource_pack.clone(),
        }
    }
    pub fn update(&mut self, tiles: &HashMap<Vec2<i64>, model::Tile>) {
        self.tiles = tiles.clone();
        let noise = &self.noise;
        let mesh = &mut self.mesh;
        mesh.clear();
        let mut append_quad =
            |pos: Vec2<i64>, h00: f32, h10: f32, h11: f32, h01: f32, a_color: Color<f32>| {
                let p = |p: Vec2<i64>, h: f32| {
                    let dv = vec2(
                        noise.get([p.x as f64, p.y as f64]) as f32,
                        noise.get([p.x as f64, p.y as f64 + 100.0]) as f32,
                    );
                    (p.map(|x| x as f32) + dv).extend(h)
                };
                let p00 = p(pos, h00);
                let p10 = p(pos + vec2(1, 0), h10);
                let p11 = p(pos + vec2(1, 1), h11);
                let p01 = p(pos + vec2(0, 1), h01);
                mesh.push(ez3d::Vertex {
                    a_pos: p00,
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_emission: 0.0,
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: p10,
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_emission: 0.0,
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: p11,
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_emission: 0.0,
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: p00,
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_emission: 0.0,
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: p11,
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_emission: 0.0,
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: p01,
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_emission: 0.0,
                    a_color,
                });
            };
        for (&tile_pos, tile) in tiles {
            let x_height = if let Some(tile_x) = tiles.get(&vec2(tile_pos.x + 1, tile_pos.y)) {
                tile_x.height
            } else {
                tile.height
            };
            let y_height = if let Some(tile_y) = tiles.get(&vec2(tile_pos.x, tile_pos.y + 1)) {
                tile_y.height
            } else {
                tile.height
            };
            let xy_height = if let Some(tile_xy) = tiles.get(&vec2(tile_pos.x + 1, tile_pos.y + 1))
            {
                tile_xy.height
            } else {
                (x_height + y_height) / 2.0
            };
            append_quad(
                tile_pos,
                tile.height,
                x_height,
                xy_height,
                y_height,
                self.resource_pack.biomes[&tile.biome].color,
            );
        }
        for (&tile_pos, _) in tiles {
            append_quad(
                tile_pos,
                0.0,
                0.0,
                0.0,
                0.0,
                Color::rgba(0.0, 0.5, 1.0, 0.5),
            );
        }
        ez3d::calc_normals(&mut self.mesh);
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        light: &light::Uniforms,
    ) {
        self.ez3d.draw(
            framebuffer,
            camera,
            light,
            &self.mesh,
            std::iter::once(ez3d::Instance {
                i_pos: vec3(0.0, 0.0, 0.0),
                i_rotation: 0.0,
                i_size: 1.0,
                i_color: Color::WHITE,
            }),
        );
    }
    pub fn get_height(&self, pos: Vec2<f32>) -> Option<f32> {
        self.intersect(camera::Ray {
            from: pos.extend(0.0),
            dir: vec3(0.0, 0.0, -1.0),
        })
        .map(|pos| pos.z)
    }
    pub fn intersect(&self, ray: camera::Ray) -> Option<Vec3<f32>> {
        let mut result: Option<(f32, Vec3<f32>)> = None;
        for face in self.mesh.chunks_exact(3) {
            let plane_pos = face[0].a_pos;
            let normal = face[0].a_normal;
            // (ray.from + ray.dir * t - plane_pos, normal) = 0
            let t = Vec3::dot(plane_pos - ray.from, normal) / Vec3::dot(ray.dir, normal);
            let p = ray.from + ray.dir * t;
            let mut consider = true;
            for i in 0..3 {
                let p1 = face[i].a_pos;
                let p2 = face[(i + 1) % 3].a_pos;
                let inside = Vec3::cross(normal, p2 - p1);
                if Vec3::dot(p - p1, inside) < 0.0 {
                    consider = false;
                    break;
                }
            }
            if consider {
                if result.is_none() || t < result.unwrap().0 {
                    result = Some((t, p));
                }
            }
        }
        result.map(|(_, p)| p)
    }
}
