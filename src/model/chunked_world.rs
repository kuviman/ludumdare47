use super::*;

pub struct ChunkedWorld {
    path: std::path::PathBuf,
    world_gen: WorldGen,
    chunk_size: Vec2<usize>,
    chunks: HashMap<Vec2<i64>, Chunk>,
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

    pub fn get_tile_f32(&self, pos: Vec2<f32>) -> Option<&Tile> {
        self.get_tile(pos.map(|x| x.floor() as i64))
    }

    pub fn get_tile(&self, pos: Vec2<i64>) -> Option<&Tile> {
        let chunk_pos = self.get_chunk_pos(pos);
        self.chunks.get(&chunk_pos).map(|chunk| &chunk.tiles[&pos])
    }

    pub fn set_load_area_for(
        &mut self,
        loader: Id,
        id_generator: &mut IdGenerator,
        area: Option<AABB<f32>>,
    ) {
        let area = area.map(|area| AABB {
            x_min: util::div_down(area.x_min.floor() as i64, self.chunk_size.x as i64),
            x_max: util::div_up(area.x_max.ceil() as i64 - 1, self.chunk_size.x as i64) + 1,
            y_min: util::div_down(area.y_min.floor() as i64, self.chunk_size.y as i64),
            y_max: util::div_up(area.y_max.ceil() as i64 - 1, self.chunk_size.y as i64) + 1,
        });
        for (&chunk_pos, chunk) in &mut self.chunks {
            if let Some(area) = &area {
                if !area.contains(chunk_pos) {
                    chunk.unload(loader);
                }
            } else {
                chunk.forget(loader);
            }
        }
        if let Some(area) = area {
            for chunk_pos in area.points() {
                let chunk = self.load_chunk(chunk_pos, id_generator);
                chunk.load(loader);
            }
        }
        self.chunks.retain(|_, chunk| chunk.is_needed());
    }
    pub fn get_updates(&mut self, loader: Id, sender: &mut dyn geng::net::Sender<ServerMessage>) {
        for chunk in self.chunks.values_mut() {
            chunk.get_updates(loader, sender);
        }
    }
}

#[derive(Deref, DerefMut)]
struct Chunk {
    #[deref]
    #[deref_mut]
    saved: util::Saved<SavedChunk>,
    area: AABB<i64>,
    version: u64,
    loaders: HashMap<Id, Option<u64>>,
}

impl Chunk {
    fn new(area: AABB<i64>, saved: util::Saved<SavedChunk>) -> Self {
        Self {
            saved,
            area,
            version: 1,
            loaders: HashMap::new(),
        }
    }
    fn is_needed(&self) -> bool {
        !self.loaders.is_empty()
    }
    fn unload(&mut self, loader: Id) {
        if let Some(version) = self.loaders.get_mut(&loader) {
            *version = None;
        }
    }
    fn forget(&mut self, loader: Id) {
        self.loaders.remove(&loader);
    }
    fn load(&mut self, loader: Id) {
        self.loaders.entry(loader).or_insert(Some(0));
    }
    fn get_updates(&mut self, loader: Id, sender: &mut dyn geng::net::Sender<ServerMessage>) {
        match self.loaders.get_mut(&loader) {
            Some(Some(version)) => {
                if *version != self.version {
                    *version = self.version;
                    sender.send(ServerMessage::UpdateTiles(self.saved.tiles.clone()));
                }
            }
            Some(None) => {
                sender.send(ServerMessage::UnloadArea(self.area));
                self.loaders.remove(&loader);
            }
            None => {}
        }
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
            let chunk_area = AABB::pos_size(
                chunk_pos * self.chunk_size.map(|x| x as i64),
                self.chunk_size.map(|x| x as i64),
            );
            let chunk_path = self
                .path
                .join("chunks")
                .join(format!("chunk_{}_{}.chunk", chunk_pos.x, chunk_pos.y));
            let saved_chunk = util::Saved::new(chunk_path, || {
                SavedChunk::generate(&self.world_gen, id_generator, chunk_area)
            });
            let chunk = Chunk::new(chunk_area, saved_chunk);
            self.chunks.insert(chunk_pos, chunk);
        }
        self.chunks.get_mut(&chunk_pos).unwrap()
    }
    fn get_chunk_pos(&self, pos: Vec2<i64>) -> Vec2<i64> {
        vec2(
            util::div_down(pos.x, self.chunk_size.x as i64),
            util::div_down(pos.y, self.chunk_size.y as i64),
        )
    }
}
