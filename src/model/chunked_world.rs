use super::*;

pub struct ChunkedWorld {
    path: std::path::PathBuf,
    world_gen: WorldGen,
    chunk_size: Vec2<usize>,
    chunks: HashMap<Vec2<i64>, Chunk>,
}

fn div_down(a: i64, b: i64) -> i64 {
    if a < 0 {
        return -div_up(-a, b);
    }
    if b < 0 {
        return -div_up(a, -b);
    }
    return a / b;
}
fn div_up(a: i64, b: i64) -> i64 {
    if a < 0 {
        return -div_down(-a, b);
    }
    if b < 0 {
        return -div_down(a, -b);
    }
    return (a + b - 1) / b;
}

impl ChunkedWorld {
    pub fn new(
        path: impl AsRef<std::path::Path>,
        chunk_size: Vec2<usize>,
        world_gen: WorldGen,
    ) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            chunk_size,
            world_gen,
            chunks: HashMap::new(),
        }
    }

    pub fn insert_item(&mut self, item: Item) -> Result<(), Item> {
        let chunk_pos = self.get_chunk_pos(item.pos.map(|x| x as i64));
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.items.insert(item.id, item);
            Ok(())
        } else {
            Err(item)
        }
    }

    pub fn remove_item(&mut self, id: Id) -> Option<Item> {
        for chunk in self.chunks.values_mut() {
            if let Some(item) = chunk.items.remove(&id) {
                return Some(item);
            }
        }
        None
    }

    pub fn get_item(&self, id: Id) -> Option<&Item> {
        for chunk in self.chunks.values() {
            if let Some(item) = chunk.items.get(&id) {
                return Some(item);
            }
        }
        None
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.chunks.values().flat_map(|chunk| chunk.items.values())
    }

    pub fn get_tile(&self, pos: Vec2<i64>) -> Option<&Tile> {
        let chunk_pos = self.get_chunk_pos(pos);
        self.chunks.get(&chunk_pos).map(|chunk| &chunk.tiles[&pos])
    }

    pub fn load_area(&mut self, id_generator: &mut IdGenerator, area: AABB<i64>) {
        let chunks_area = AABB {
            x_min: div_down(area.x_min, self.chunk_size.x as i64),
            x_max: div_up(area.x_max - 1, self.chunk_size.x as i64) + 1,
            y_min: div_down(area.y_min, self.chunk_size.y as i64),
            y_max: div_up(area.y_max - 1, self.chunk_size.y as i64) + 1,
        };
        for chunk_pos in chunks_area.points() {
            self.load_chunk(chunk_pos, id_generator);
        }
    }
}

#[derive(Deref, DerefMut)]
struct Chunk {
    #[deref]
    saved: util::Saved<SavedChunk>,
}

impl Chunk {
    fn new(saved: util::Saved<SavedChunk>) -> Self {
        Self { saved }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SavedChunk {
    tiles: HashMap<Vec2<i64>, Tile>,
    items: HashMap<Id, Item>,
}

impl SavedChunk {
    fn generate(world_gen: &WorldGen, id_generator: &mut IdGenerator, area: AABB<i64>) -> Self {
        let mut tiles = HashMap::new();
        let mut items = HashMap::new();
        for pos in area.points() {
            let (tile, item) = world_gen.generate_tile(id_generator, pos);
            tiles.insert(pos, tile);
            if let Some(item) = item {
                items.insert(item.id, item);
            }
        }
        Self { tiles, items }
    }
}

impl ChunkedWorld {
    fn load_chunk(&mut self, chunk_pos: Vec2<i64>, id_generator: &mut IdGenerator) -> &mut Chunk {
        if !self.chunks.contains_key(&chunk_pos) {
            let chunk_path = self
                .path
                .join("chunks")
                .join(format!("chunk_{}_{}.chunk", chunk_pos.x, chunk_pos.y));
            let saved_chunk = util::Saved::new(chunk_path, || {
                let chunk_size = self.chunk_size.map(|x| x as i64);
                SavedChunk::generate(
                    &self.world_gen,
                    id_generator,
                    AABB::pos_size(chunk_pos * chunk_size, chunk_size),
                )
            });
            let chunk = Chunk::new(saved_chunk);
            self.chunks.insert(chunk_pos, chunk);
        }
        self.chunks.get_mut(&chunk_pos).unwrap()
    }
    fn get_chunk_pos(&self, pos: Vec2<i64>) -> Vec2<i64> {
        vec2(
            div_down(pos.x, self.chunk_size.x as i64),
            div_down(pos.y, self.chunk_size.y as i64),
        )
    }
}
