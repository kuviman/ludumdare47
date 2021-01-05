use super::*;

impl Model {
    pub fn spawn_player(&mut self) -> Id {
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
    // TODO: delete, should be done in better way
    fn is_empty_tile(&self, pos: Vec2<i64>) -> bool {
        !self
            .chunked_world
            .items()
            .any(|item| pos == item.pos.map(|x| x as i64))
            && !self
                .players
                .values()
                .any(|player| pos == player.pos.map(|x| x as i64))
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
    fn get_spawnable_pos(&mut self, origin: Vec2<i64>, search_range: usize) -> Option<Vec2<i64>> {
        let search_range = search_range as i64;
        let area = AABB::from_corners(
            origin - vec2(search_range, search_range),
            origin + vec2(search_range, search_range) + vec2(1, 1),
        );
        self.chunked_world.load_area(&mut self.id_generator, area);
        area.points()
            .filter(|&pos| self.is_spawnable(pos))
            .min_by_key(|&pos| (pos - origin).map(|x| x as f32).len() as i64) // Meh
    }
}
