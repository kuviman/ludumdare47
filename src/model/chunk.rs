use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Chunk {
    pub tile_map: HashMap<Vec2<i64>, Tile>,
    pub items: HashMap<Id, Item>,
    pub is_loaded: bool,
}

impl Chunk {
    pub fn load(world_name: &str, chunk_pos: Vec2<i64>) -> Result<Chunk, anyhow::Error> {
        let mut chunk: Chunk = bincode::deserialize_from(std::io::BufReader::new(
            std::fs::File::open(Self::save_file_path(world_name, chunk_pos))?,
        ))?;
        chunk.is_loaded = true;
        Ok(chunk)
    }
    pub fn save(&self, world_name: &str, chunk_pos: Vec2<i64>) -> Result<(), anyhow::Error> {
        bincode::serialize_into(
            std::io::BufWriter::new(std::fs::File::create(Self::save_file_path(
                world_name, chunk_pos,
            ))?),
            self,
        )?;
        Ok(())
    }
    fn save_file_path(world_name: &str, chunk_pos: Vec2<i64>) -> String {
        format!(
            "saves/{}/chunks/chunk_{}_{}.chunk",
            world_name, chunk_pos.x, chunk_pos.y
        )
    }
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
