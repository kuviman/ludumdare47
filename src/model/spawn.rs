use super::*;

impl Model {
    pub fn spawn_player(&mut self) -> Id {
        let player_id = self.id_generator.gen();
        if let Some(pos) = self.get_spawnable_pos(player_id, vec2(0.0, 0.0), self.rules.spawn_area)
        {
            let player = Player {
                id: player_id,
                pos: pos.map(|x| x as f32),
                radius: 0.5,
                interaction_range: self.rules.player_interaction_range,
                item: None,
                colors: PlayerColors::new(),
                action: None,
                load_area: AABB::pos_size(pos.map(|x| x as f32), vec2(0.0, 0.0)),
            };
            self.sounds.insert(player.id, vec![]);
            self.players.insert(player.id, player);
        } else {
            error!("Did not find spawnable position"); // TODO
        }
        player_id
    }
    // TODO: delete, should be done in better way
    fn is_empty_tile(&self, pos: Vec2<i64>) -> bool {
        !self
            .chunked_world
            .items()
            .any(|item| pos == get_tile_pos(item.pos))
            && !self
                .players
                .values()
                .any(|player| pos == get_tile_pos(player.pos))
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
    ) -> Option<Vec2<i64>> {
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
    }
}
