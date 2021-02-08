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

    pub fn insert_entity(&mut self, entity: Entity) -> Result<(), Entity> {
        let chunk_pos = self.get_chunk_pos(get_tile_pos(entity.pos));
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.borrow_mut().entities.insert(entity.id, entity);
            Ok(())
        } else {
            Err(entity)
        }
    }

    pub fn update_entity(&mut self, id: Id) {
        if let Some((entity, chunk_pos)) = self.get_entity_chunk_pos(id) {
            let entity_chunk_pos = self.get_chunk_pos(get_tile_pos(entity.pos));
            if chunk_pos != entity_chunk_pos {
                let entity = self
                    .chunks
                    .get_mut(&chunk_pos)
                    .unwrap()
                    .borrow_mut()
                    .entities
                    .remove(&id)
                    .unwrap();
                if let Some(entity_chunk) = self.chunks.get_mut(&entity_chunk_pos) {
                    entity_chunk.borrow_mut().entities.insert(entity.id, entity);
                } else {
                    unreachable!();
                }
            }
        }
    }

    pub fn remove_entity(&mut self, id: Id) -> Option<Entity> {
        for chunk in self.chunks.values_mut() {
            if chunk.entities.contains_key(&id) {
                return Some(chunk.borrow_mut().entities.remove(&id).unwrap());
            }
        }
        None
    }

    pub fn get_entity(&self, id: Id) -> Option<&Entity> {
        for chunk in self.chunks.values() {
            if let Some(entity) = chunk.entities.get(&id) {
                return Some(entity);
            }
        }
        None
    }

    pub fn get_entity_mut(&mut self, id: Id) -> Option<&mut Entity> {
        for chunk in self.chunks.values_mut() {
            if let Some(entity) = chunk.borrow_mut().entities.get_mut(&id) {
                return Some(entity);
            }
        }
        None
    }

    pub fn get_entity_chunk_pos(&self, id: Id) -> Option<(&Entity, Vec2<i64>)> {
        for (&chunk_pos, chunk) in &self.chunks {
            if let Some(entity) = chunk.entities.get(&id) {
                return Some((entity, chunk_pos));
            }
        }
        None
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.chunks
            .values()
            .flat_map(|chunk| chunk.entities.values())
    }

    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.chunks
            .values_mut()
            .flat_map(|chunk| chunk.borrow_mut().entities.values_mut())
    }

    pub fn find_range(
        &self,
        pos: Vec2<f32>,
        range: f32,
        predicate: impl Fn(&Entity) -> bool,
    ) -> Vec<&Entity> {
        let mut entities = Vec::new();
        let chunk_dist =
            (vec2(range, range) / self.chunk_size.map(|x| x as f32)).map(|x| x.ceil() as i64);
        let chunk_pos = self.get_chunk_pos(get_tile_pos(pos));
        for y in chunk_pos.y - chunk_dist.y..chunk_pos.y + chunk_dist.y {
            for x in chunk_pos.x - chunk_dist.x..chunk_pos.x + chunk_dist.x {
                if let Some(chunk) = self.chunks.get(&vec2(x, y)) {
                    for entity in chunk.entities.values().filter(|e| predicate(e)) {
                        let delta = pos - entity.pos;
                        if delta.x * delta.x + delta.y * delta.y <= range * range {
                            entities.push(entity);
                        }
                    }
                }
            }
        }
        entities
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
        self.chunks.retain(|_, chunk| chunk.has_loaders());
    }
    pub fn get_updates(&mut self, loader: Id, sender: &mut dyn geng::net::Sender<ServerMessage>) {
        let mut removes = Vec::new();
        let mut inserts = Vec::new();
        for chunk in self.chunks.values_mut() {
            chunk.get_updates(loader, &mut removes, &mut inserts);
        }
        for message in removes {
            sender.send(message);
        }
        for message in inserts {
            sender.send(message);
        }
    }
}

#[derive(Deref, DerefMut)]
struct Chunk {
    #[deref]
    #[deref_mut]
    inner: util::Loaded<util::Saved<SavedChunk>>,
    area: AABB<i64>,
}

impl Chunk {
    fn new(area: AABB<i64>, saved: util::Saved<SavedChunk>) -> Self {
        Self {
            inner: util::Loaded::new(saved),
            area,
        }
    }
    fn get_updates(
        &mut self,
        loader: Id,
        removes: &mut Vec<ServerMessage>,
        inserts: &mut Vec<ServerMessage>,
    ) {
        match self.get_update(loader) {
            Some(util::LoadedUpdate::Update) => {
                inserts.push(ServerMessage::UpdateTiles(self.tiles.clone()))
            }
            Some(util::LoadedUpdate::Unload) => removes.push(ServerMessage::UnloadArea(self.area)),
            None => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedChunk {
    chunk_pos: Vec2<i64>,
    tiles: HashMap<Vec2<i64>, Tile>,
    entities: HashMap<Id, Entity>,
}

impl SavedChunk {
    fn generate(
        chunk_pos: Vec2<i64>,
        world_gen: &WorldGen,
        id_generator: &mut IdGenerator,
        area: AABB<i64>,
    ) -> Self {
        let mut tiles = HashMap::new();
        let mut entities = HashMap::new();
        for pos in area.points() {
            let (tile, entity_type) = world_gen.generate_tile(pos);
            tiles.insert(pos, tile);
            if let Some(entity_type) = entity_type {
                let entity = Entity::new(
                    &entity_type,
                    &world_gen.entity_properties[&entity_type],
                    pos.map(|x| x as f32),
                    id_generator.gen(),
                );
                entities.insert(entity.id, entity);
            }
        }
        Self {
            chunk_pos,
            tiles,
            entities,
        }
    }
}

impl ChunkedWorld {
    fn load_chunk(&mut self, chunk_pos: Vec2<i64>, id_generator: &mut IdGenerator) -> &mut Chunk {
        if !self.chunks.contains_key(&chunk_pos) {
            debug!("Loading chunk {}", chunk_pos);
            let chunk_area = AABB::pos_size(
                chunk_pos * self.chunk_size.map(|x| x as i64),
                self.chunk_size.map(|x| x as i64),
            );
            let chunk_path = self
                .path
                .join("chunks")
                .join(format!("chunk_{}_{}.chunk", chunk_pos.x, chunk_pos.y));
            let saved_chunk = util::Saved::new(chunk_path, || {
                info!("Generating chunk {}", chunk_pos);
                SavedChunk::generate(chunk_pos, &self.world_gen, id_generator, chunk_area)
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
