use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Chunk {
    pub tile_map: HashMap<Vec2<i64>, Tile>,
    pub items: HashMap<Id, Item>,
}

impl Model {
    pub fn local_to_global_pos(
        chunk_size: Vec2<usize>,
        chunk_pos: Vec2<i64>,
        pos: Vec2<i64>,
    ) -> Vec2<i64> {
        vec2(
            chunk_pos.x * chunk_size.x as i64 + pos.x,
            chunk_pos.y * chunk_size.y as i64 + pos.y,
        )
    }
    pub fn get_chunk_pos(&self, pos: Vec2<i64>) -> Vec2<i64> {
        let x = if pos.x >= 0 {
            pos.x / self.chunk_size.x as i64
        } else {
            (pos.x + 1) / self.chunk_size.x as i64 - 1
        };
        let y = if pos.y >= 0 {
            pos.y / self.chunk_size.y as i64
        } else {
            (pos.y + 1) / self.chunk_size.y as i64 - 1
        };
        vec2(x, y)
    }
}
