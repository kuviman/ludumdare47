use super::*;

pub struct TileMesh {
    pub tiles: HashMap<Vec2<usize>, model::Tile>,
    pub height_map: Vec<Vec<f32>>,
    pub mesh: ugli::VertexBuffer<ez3d::Vertex>,
}

impl TileMesh {
    pub fn new(
        geng: &Rc<Geng>,
        tiles: &[model::Tile],
        height_map: &Vec<Vec<f32>>,
        noise: &dyn NoiseFn<[f64; 2]>,
    ) -> Self {
        let mut mesh = Vec::new();
        let tiles: HashMap<Vec2<usize>, model::Tile> =
            tiles.iter().map(|tile| (tile.pos, tile.clone())).collect();
        let mut append_quad =
            |pos: Vec2<usize>, h00: f32, h10: f32, h11: f32, h01: f32, a_color: Color<f32>| {
                let p = |p: Vec2<usize>, h: f32| {
                    let dv = vec2(
                        noise.get([p.x as f64, p.y as f64]) as f32,
                        noise.get([p.y as f64, p.y as f64 + 100.0]) as f32,
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
        for tile in tiles.values() {
            append_quad(
                tile.pos,
                height_map[tile.pos.x][tile.pos.y],
                height_map[tile.pos.x + 1][tile.pos.y],
                height_map[tile.pos.x + 1][tile.pos.y + 1],
                height_map[tile.pos.x][tile.pos.y + 1],
                match tile.ground_type {
                    model::GroundType::Water => Color::rgb(0.8, 0.8, 0.0),
                    model::GroundType::Grass => Color::rgb(0.0, 0.8, 0.0),
                    model::GroundType::Sand => Color::YELLOW,
                },
            );
        }
        for tile in tiles.values() {
            append_quad(
                tile.pos,
                0.0,
                0.0,
                0.0,
                0.0,
                Color::rgba(0.0, 0.5, 1.0, 0.5),
            );
        }
        ez3d::calc_normals(&mut mesh);
        Self {
            tiles,
            height_map: height_map.clone(),
            mesh: ugli::VertexBuffer::new_dynamic(geng.ugli(), mesh),
        }
    }
    pub fn get_height(&self, pos: Vec2<f32>) -> Option<f32> {
        let pos_f = pos.map(|x| x.fract());
        let pos = pos.map(|x| x as usize);
        let h00 = self.height_map[pos.x][pos.y];
        let h10 = self.height_map[pos.x + 1][pos.y];
        let h11 = self.height_map[pos.x + 1][pos.y + 1];
        let h01 = self.height_map[pos.x][pos.y + 1];
        Some(if pos_f.y < pos_f.x {
            h00 * (1.0 - pos_f.x) + (h10 * (1.0 - pos_f.y) + h11 * pos_f.y) * pos_f.x
        } else {
            h00 * (1.0 - pos_f.y) + (h01 * (1.0 - pos_f.x) + h11 * pos_f.x) * pos_f.y
        })
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
