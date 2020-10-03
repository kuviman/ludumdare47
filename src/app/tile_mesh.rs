use super::*;

pub struct TileMesh {
    pub tiles: HashMap<Vec2<usize>, model::Tile>,
    pub height_map: Vec<Vec<f32>>,
    pub mesh: ugli::VertexBuffer<ez3d::Vertex>,
}

impl TileMesh {
    pub fn new(geng: &Rc<Geng>, tiles: &[model::Tile], height_map: &Vec<Vec<f32>>) -> Self {
        let mut mesh = Vec::new();
        let tiles: HashMap<Vec2<usize>, model::Tile> =
            tiles.iter().map(|tile| (tile.pos, tile.clone())).collect();
        let mut append_quad =
            |p: Vec2<usize>, h00: f32, h10: f32, h11: f32, h01: f32, a_color: Color<f32>| {
                let p = p.map(|x| x as f32);
                mesh.push(ez3d::Vertex {
                    a_pos: vec3(p.x, p.y, h00),
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: vec3(p.x + 1.0, p.y, h10),
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: vec3(p.x + 1.0, p.y + 1.0, h11),
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: vec3(p.x, p.y, h00),
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: vec3(p.x + 1.0, p.y + 1.0, h11),
                    a_normal: vec3(0.0, 0.0, 0.0),
                    a_color,
                });
                mesh.push(ez3d::Vertex {
                    a_pos: vec3(p.x, p.y + 1.0, h01),
                    a_normal: vec3(0.0, 0.0, 0.0),
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
}
