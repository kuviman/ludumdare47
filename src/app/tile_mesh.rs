use super::*;

pub struct TileMesh {
    pub tiles: HashMap<Vec2<usize>, model::Tile>,
    pub mesh: ugli::VertexBuffer<ez3d::Vertex>,
}

impl TileMesh {
    pub fn new(geng: &Rc<Geng>, tiles: &[model::Tile]) -> Self {
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
        let mut append = |tile00: &model::Tile| -> Option<()> {
            let tile01 = tiles.get(&(tile00.pos + vec2(0, 1)))?;
            let tile11 = tiles.get(&(tile00.pos + vec2(1, 1)))?;
            let tile10 = tiles.get(&(tile00.pos + vec2(1, 0)))?;
            let p00 = tile00.pos.map(|x| x as f32).extend(tile00.height);
            let p01 = tile01.pos.map(|x| x as f32).extend(tile01.height);
            let p11 = tile11.pos.map(|x| x as f32).extend(tile11.height);
            let p10 = tile10.pos.map(|x| x as f32).extend(tile10.height);
            append_quad(
                tile00.pos,
                tile00.height,
                tile10.height,
                tile11.height,
                tile01.height,
                match tile00.ground_type {
                    model::GroundType::Water => Color::rgb(0.8, 0.8, 0.0),
                    model::GroundType::Sand => Color::YELLOW,
                },
            );
            Some(())
        };
        for tile in tiles.values() {
            append(tile);
        }
        for tile in tiles.values() {
            if tiles.get(&(tile.pos + vec2(0, 1))).is_some()
                && tiles.get(&(tile.pos + vec2(1, 1))).is_some()
                && tiles.get(&(tile.pos + vec2(1, 0))).is_some()
            {
                append_quad(
                    tile.pos,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    Color::rgba(0.0, 0.5, 1.0, 0.5),
                );
            }
        }
        ez3d::calc_normals(&mut mesh);
        Self {
            tiles,
            mesh: ugli::VertexBuffer::new_dynamic(geng.ugli(), mesh),
        }
    }
    pub fn get_height(&self, pos: Vec2<f32>) -> Option<f32> {
        let pos_f = pos.map(|x| x.fract());
        let pos = pos.map(|x| x as usize);
        let h00 = self.tiles.get(&pos)?.height;
        let h01 = self.tiles.get(&(pos + vec2(0, 1)))?.height;
        let h11 = self.tiles.get(&(pos + vec2(1, 1)))?.height;
        let h10 = self.tiles.get(&(pos + vec2(1, 0)))?.height;
        Some(if pos_f.y < pos_f.x {
            h00 * (1.0 - pos_f.x) + (h10 * (1.0 - pos_f.y) + h11 * pos_f.y) * pos_f.x
        } else {
            h00 * (1.0 - pos_f.y) + (h01 * (1.0 - pos_f.x) + h11 * pos_f.x) * pos_f.y
        })
    }
}
