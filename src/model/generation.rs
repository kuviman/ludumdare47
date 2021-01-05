use super::*;

impl Model {
    pub fn new_player(&mut self) -> Id {
        let player_id;
        if let Some(pos) = self.get_spawnable_pos(vec2(0, 0), self.rules.spawn_area) {
            let player = Player {
                id: self.id_generator.gen(),
                pos: pos.map(|x| x as f32),
                radius: 0.5,
                interaction_range: self.rules.player_interaction_range,
                item: None,
                colors: PlayerColors::new(),
                action: None,
                load_area: AABB::pos_size(pos.map(|x| x as f32), vec2(0.0, 0.0)),
            };
            player_id = player.id;
            self.sounds.insert(player.id, vec![]);
            self.players.insert(player.id, player);
        } else {
            error!("Did not find spawnable position");
            player_id = self.id_generator.gen(); // TODO
        }
        player_id
    }
    pub fn spawn_item(&mut self, item_type: ItemType, pos: Vec2<f32>) {
        let item = Item {
            id: self.id_generator.gen(),
            pos,
            size: self.resource_pack.items[&item_type].size,
            item_type,
        };
        self.chunked_world.insert_item(item);
    }
    pub fn remove_item_id(&mut self, id: Id) -> Option<Item> {
        self.chunked_world.remove_item(id)
    }
    pub fn get_tile(&self, pos: Vec2<i64>) -> Option<&Tile> {
        self.chunked_world.get_tile(pos)
    }
    fn is_spawnable_tile(&self, pos: Vec2<i64>) -> bool {
        self.resource_pack.biomes[&self.get_tile(pos).unwrap().biome].spawnable
            && self.is_empty_tile(pos)
    }
    fn get_spawnable_pos(&mut self, origin: Vec2<i64>, search_range: usize) -> Option<Vec2<i64>> {
        let search_range = search_range as i64;
        let area = AABB::from_corners(
            origin - vec2(search_range, search_range),
            origin + vec2(search_range, search_range) + vec2(1, 1),
        );
        self.chunked_world.load_area(&mut self.id_generator, area);
        area.points()
            .filter(|pos| self.is_spawnable_tile(*pos))
            .min_by_key(|&pos| (pos - origin).map(|x| x as f32).len() as i64) // Meh
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeGeneration {
    pub collidable: bool,
    pub spawnable: bool,
    pub parameters: HashMap<GenerationParameter, (f32, f32)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationParameter(pub String);
