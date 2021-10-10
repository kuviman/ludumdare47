use super::*;

pub struct TileMesh {
    geng: Geng,
    chunk_size: i64,
    ez3d: Rc<Ez3D>,
    noise: Rc<::noise::OpenSimplex>,
    chunks: HashMap<Vec2<i64>, Chunk>,
    water_mesh: ugli::VertexBuffer<ez3d::Vertex>,
    resource_pack: Rc<ResourcePack>,
}

struct Chunk {
    noise: Rc<::noise::OpenSimplex>,
    tiles: HashMap<Vec2<i64>, model::Tile>,
    mesh: ugli::VertexBuffer<ez3d::Vertex>,
    resource_pack: Rc<ResourcePack>,
}

impl Chunk {
    fn get_faces(&self, pos: Vec2<i64>) -> Option<[[Vec3<f32>; 3]; 2]> {
        let v = |pos: Vec2<i64>| -> Option<Vec3<f32>> {
            let tile = self.tiles.get(&pos)?;
            let height = tile.world_parameters[&model::WorldParameter("Height".to_owned())];
            let shift = vec2(
                self.noise.get([pos.x as f64, pos.y as f64]) as f32 / 0.55 / 2.1,
                self.noise.get([pos.x as f64, pos.y as f64 + 100.0]) as f32 / 0.55 / 2.1,
            );
            let pos = pos.map(|x| x as f32) + shift;
            let pos = pos.extend(height);
            Some(pos)
        };

        let vs = [
            v(pos)?,
            v(pos + vec2(1, 0))?,
            v(pos + vec2(1, 1))?,
            v(pos + vec2(0, 1))?,
        ];

        Some([[vs[0], vs[1], vs[2]], [vs[0], vs[2], vs[3]]])
    }
    fn all_faces(&self) -> impl Iterator<Item = [Vec3<f32>; 3]> + '_ {
        self.mesh.chunks_exact(3).map(|face| {
            let p0 = face[0].a_pos;
            let p1 = face[1].a_pos;
            let p2 = face[2].a_pos;
            [p0, p1, p2]
        })
    }
    fn unload(&mut self, area: AABB<i64>) {
        let updated = self.tiles.keys().any(|&pos| area.contains(pos));
        self.tiles.retain(|&pos, _| !area.contains(pos));
        if updated {
            self.update_mesh();
        }
    }
    fn update_mesh(&mut self) {
        let mut mesh = self
            .tiles
            .keys()
            .filter_map(|&pos| {
                let tile = self.tiles.get(&pos)?;
                let color = self.resource_pack.biomes[&tile.biome].color;
                Some(
                    util::iter2(self.get_faces(pos)?).flat_map(move |face: [Vec3<f32>; 3]| {
                        util::iter3(face).map(move |vertex: Vec3<f32>| ez3d::Vertex {
                            a_pos: vertex,
                            a_normal: vec3(0.0, 0.0, 0.0),
                            a_color: color,
                            a_emission: 0.0,
                        })
                    }),
                )
            })
            .flatten()
            .collect::<Vec<ez3d::Vertex>>();
        ez3d::calc_normals(&mut mesh);
        *self.mesh = mesh;
    }
}

fn intersect(face: &[Vec3<f32>; 3], ray: camera::Ray) -> Option<f32> {
    let plane_pos = face[0];
    let normal = Vec3::cross(face[1] - face[0], face[2] - face[0]);
    // (ray.from + ray.dir * t - plane_pos, normal) = 0
    let t = Vec3::dot(plane_pos - ray.from, normal) / Vec3::dot(ray.dir, normal);
    let p = ray.from + ray.dir * t;
    for i in 0..3 {
        let p1 = face[i];
        let p2 = face[(i + 1) % 3];
        let inside = Vec3::cross(normal, p2 - p1);
        if Vec3::dot(p - p1, inside) < -1e-3 {
            return None;
        }
    }
    Some(t)
}

impl TileMesh {
    pub fn new(geng: &Geng, ez3d: &Rc<Ez3D>, resource_pack: &Rc<ResourcePack>) -> Self {
        Self {
            geng: geng.clone(),
            chunk_size: 16,
            ez3d: ez3d.clone(),
            noise: Rc::new(::noise::OpenSimplex::new()),
            chunks: HashMap::new(),
            water_mesh: ugli::VertexBuffer::new_static(geng.ugli(), {
                let inf = 1e3;
                vec![
                    ez3d::Vertex {
                        a_pos: vec3(-inf, -inf, 0.0),
                        a_normal: vec3(0.0, 0.0, 1.0),
                        a_color: Color::WHITE,
                        a_emission: 0.0,
                    },
                    ez3d::Vertex {
                        a_pos: vec3(inf, -inf, 0.0),
                        a_normal: vec3(0.0, 0.0, 1.0),
                        a_color: Color::WHITE,
                        a_emission: 0.0,
                    },
                    ez3d::Vertex {
                        a_pos: vec3(inf, inf, 0.0),
                        a_normal: vec3(0.0, 0.0, 1.0),
                        a_color: Color::WHITE,
                        a_emission: 0.0,
                    },
                    ez3d::Vertex {
                        a_pos: vec3(-inf, inf, 0.0),
                        a_normal: vec3(0.0, 0.0, 1.0),
                        a_color: Color::WHITE,
                        a_emission: 0.0,
                    },
                ]
            }),
            resource_pack: resource_pack.clone(),
        }
    }
    fn get_chunk_pos(&self, pos: Vec2<i64>) -> Vec2<i64> {
        vec2(
            util::div_down(pos.x, self.chunk_size),
            util::div_down(pos.y, self.chunk_size),
        )
    }
    pub fn update(&mut self, tiles: &HashMap<Vec2<i64>, model::Tile>) {
        let mut updated_chunks = HashSet::new();
        for (&pos, tile) in tiles {
            for dx in -1..=0 {
                for dy in -1..=0 {
                    let update_in = pos + vec2(dx, dy);
                    let chunk_pos = self.get_chunk_pos(update_in);
                    if !self.chunks.contains_key(&chunk_pos) {
                        self.chunks.insert(
                            chunk_pos,
                            Chunk {
                                noise: self.noise.clone(),
                                tiles: HashMap::new(),
                                mesh: ugli::VertexBuffer::new_static(self.geng.ugli(), vec![]),
                                resource_pack: self.resource_pack.clone(),
                            },
                        );
                    }
                    updated_chunks.insert(chunk_pos);
                    self.chunks
                        .get_mut(&chunk_pos)
                        .unwrap()
                        .tiles
                        .insert(pos, tile.clone());
                }
            }
        }
        for chunk_pos in updated_chunks {
            self.chunks.get_mut(&chunk_pos).unwrap().update_mesh();
        }
    }
    pub fn unload(&mut self, area: AABB<i64>) {
        for chunk in self.chunks.values_mut() {
            chunk.unload(area);
        }
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        light: &light::Uniforms,
    ) {
        for chunk in self.chunks.values() {
            self.ez3d.draw(
                framebuffer,
                camera,
                light,
                &chunk.mesh,
                std::iter::once(ez3d::Instance {
                    i_pos: vec3(0.0, 0.0, 0.0),
                    i_rotation: 0.0,
                    i_size: 1.0,
                    i_color: Color::WHITE,
                }),
            );
        }
        self.ez3d.draw_with(
            framebuffer,
            camera,
            light,
            &self.water_mesh,
            std::iter::once(ez3d::Instance {
                i_pos: vec3(0.0, 0.0, 0.0),
                i_rotation: 0.0,
                i_size: 1.0,
                i_color: Color::rgba(0.0, 0.5, 1.0, 0.5),
            }),
            ugli::DrawMode::TriangleFan,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                depth_func: Some(default()),
                cull_face: Some(ugli::CullFace::Back),
                ..default()
            },
        );
    }
    fn get_faces(&self, pos: Vec2<i64>) -> Option<[[Vec3<f32>; 3]; 2]> {
        self.chunks.get(&self.get_chunk_pos(pos))?.get_faces(pos)
    }
    fn all_faces(&self) -> impl Iterator<Item = [Vec3<f32>; 3]> + '_ {
        self.chunks.values().flat_map(|chunk| chunk.all_faces())
    }
    pub fn get_height(&self, pos: Vec2<f32>) -> Option<f32> {
        let ray = camera::Ray {
            from: pos.extend(0.0),
            dir: vec3(0.0, 0.0, -1.0),
        };
        let tile = pos.map(|x| x.floor() as i64);
        let faces = (-1..=1)
            .flat_map(move |dx| (-1..=1).filter_map(move |dy| self.get_faces(tile + vec2(dx, dy))))
            .flat_map(|faces| util::iter2(faces));
        let t = faces
            .filter_map(|face| intersect(&face, ray))
            .min_by_key(|&t| r32(t))?;
        Some((ray.from + ray.dir * t).z)
    }
    pub fn intersect(&self, ray: camera::Ray) -> Option<Vec3<f32>> {
        let mut result: Option<(f32, Vec3<f32>)> = None;
        for face in self.all_faces() {
            if let Some(t) = intersect(&face, ray) {
                let p = ray.from + ray.dir * t;
                if result.is_none() || t < result.unwrap().0 {
                    result = Some((t, p));
                }
            }
        }
        result.map(|(_, p)| p)
    }
}
