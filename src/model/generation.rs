use super::*;

impl Model {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeGeneration {
    pub collidable: bool,
    pub spawnable: bool,
    pub parameters: HashMap<GenerationParameter, (f32, f32)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationParameter(pub String);
