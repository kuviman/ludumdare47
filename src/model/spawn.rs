use super::*;

impl Model {
    pub fn spawn_player(&mut self) -> Id {
        let player_id = self.id_generator.gen();
        if let Some(pos) = self.get_spawnable_pos(player_id, vec2(0.0, 0.0), self.rules.spawn_area)
        {
            let entity_type = EntityType("Player".to_owned());
            let mut components = self.resource_pack.entity_components[&entity_type].clone();
            components.pos = Some(pos);
            let mut entity = Entity::new(&entity_type, components, player_id);
            let player = entity.load_area.as_mut().unwrap();
            player.load_area = AABB::pos_size(pos.map(|x| x as f32), vec2(0.0, 0.0));
            if let Some(CompRenderable::Player { colors }) = entity.renderable.as_mut() {
                *colors = PlayerColors::new();
            }

            self.sounds.insert(player_id, vec![]);
            self.chunked_world.insert_entity(entity).unwrap();
        } else {
            error!("Did not find spawnable position"); // TODO
        }
        player_id
    }
    // TODO: delete, should be done in better way
    fn is_empty_tile(&self, pos: Vec2<i64>) -> bool {
        !self
            .chunked_world
            .entities()
            .any(|entity| entity.pos.is_some() && pos == get_tile_pos(entity.pos.unwrap()))
            && !self
                .chunked_world
                .entities() // TODO: don't check every entity
                .any(|player| pos == get_tile_pos(player.pos.unwrap()))
    }
    fn is_spawnable(&self, pos: Vec2<i64>) -> bool {
        match self.chunked_world.get_tile(pos) {
            Some(tile) => {
                self.resource_pack.biome_properties[&tile.biome].spawnable
                    && self.is_empty_tile(pos)
            }
            None => false,
        }
    }
    fn get_spawnable_pos(
        &mut self,
        player_id: Id,
        origin: Vec2<f32>,
        search_range: f32,
    ) -> Option<Vec2<f32>> {
        let area = AABB::from_corners(
            origin - vec2(search_range, search_range),
            origin + vec2(search_range, search_range),
        );
        self.chunked_world
            .set_load_area_for(player_id, &mut self.id_generator, Some(area));
        area.map(|x| x as i64)
            .points()
            .filter(|&pos| self.is_spawnable(pos))
            .min_by_key(|&pos| r32((pos.map(|x| x as f32 + 0.5) - origin).len()))
            .map(|x| x.map(|x| x as f32)) //TODO: Allow non-integer coords
    }
}
