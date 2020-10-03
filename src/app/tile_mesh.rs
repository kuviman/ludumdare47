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
        let mut append = |tile00: &model::Tile| -> Option<()> {
            let tile01 = tiles.get(&(tile00.pos + vec2(0, 1)))?;
            let tile11 = tiles.get(&(tile00.pos + vec2(1, 1)))?;
            let tile10 = tiles.get(&(tile00.pos + vec2(1, 0)))?;
            let a_color = match tile00.ground_type {
                model::GroundType::Water => Color::BLUE,
                model::GroundType::Sand => Color::YELLOW,
            };
            let p00 = tile00.pos.map(|x| x as f32).extend(tile00.height);
            let p01 = tile01.pos.map(|x| x as f32).extend(tile01.height);
            let p11 = tile11.pos.map(|x| x as f32).extend(tile11.height);
            let p10 = tile10.pos.map(|x| x as f32).extend(tile10.height);
            mesh.push(ez3d::Vertex {
                a_pos: p00,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p10,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p11,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p00,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p11,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p01,
                a_color,
            });
            Some(())
        };
        for tile00 in tiles.values() {
            append(tile00);
        }
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
            h00 * (1.0 - pos_f.x) + (h10 * (1.0 - pos_f.y) + h11 * pos_f.y) * pos_f.x / 2.0
        } else {
            h00 * (1.0 - pos_f.y) + (h01 * (1.0 - pos_f.x) + h11 * pos_f.x) * pos_f.y / 2.0
        })
    }
}
