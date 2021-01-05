use super::*;

impl Model {
    pub fn tick(&mut self) {
        let ids: Vec<Id> = self.players.keys().copied().collect();
        for id in ids {
            let mut player = self.players.get(&id).unwrap().clone();
            self.player_action(&mut player);

            // Collide with items
            for item in self.chunked_world.items() {
                if !self.resource_pack.items[&item.item_type].traversable {
                    let dir = player.pos - item.pos;
                    let distance = dir.len();
                    if distance <= player.radius + item.size {
                        let penetration = player.radius + item.size - distance;
                        let normal = dir / distance;
                        player.pos += normal * penetration;
                    }
                }
            }

            // Collide with players
            if let Some((normal, penetration)) = self.players.values().find_map(|e| {
                if e.id == player.id {
                    return None;
                }

                let dir = player.pos - e.pos;
                let distance = dir.len();
                if distance <= player.radius {
                    let penetration = player.radius + e.radius - distance;
                    let normal = dir / distance;
                    Some((normal, penetration))
                } else {
                    None
                }
            }) {
                player.pos += normal * penetration;
            }

            // Collide with tiles
            for x in (-player.radius.ceil() as i64)..(player.radius.ceil() as i64 + 1) {
                for y in (-player.radius.ceil() as i64)..(player.radius.ceil() as i64 + 1) {
                    let pos = vec2(x, y) + player.pos.map(|x| x as i64);
                    if let Some((normal, penetration)) = match self.get_tile(pos) {
                        Some(tile) => {
                            if self.resource_pack.biomes[&tile.biome].collidable {
                                Self::collide(player.pos, player.radius, pos.map(|x| x as f32), 1.0)
                            } else {
                                None
                            }
                        }
                        None => None,
                    } {
                        player.pos += normal * penetration;
                    }
                }
            }

            *self.players.get_mut(&id).unwrap() = player;
        }
    }
    fn collide(
        circle_pos: Vec2<f32>,
        circle_radius: f32,
        tile_pos: Vec2<f32>,
        tile_size: f32,
    ) -> Option<(Vec2<f32>, f32)> {
        let up = circle_pos.y - tile_pos.y - tile_size;
        let down = tile_pos.y - circle_pos.y;
        let right = circle_pos.x - tile_pos.x - tile_size;
        let left = tile_pos.x - circle_pos.x;

        let (dy, ny) = if up.abs() < down.abs() {
            (up, 1.0)
        } else {
            (down, -1.0)
        };
        let (dx, nx) = if right.abs() < left.abs() {
            (right, 1.0)
        } else {
            (left, -1.0)
        };

        // Find direction and distance from the tile to the center point
        let (normal, distance) = if dx <= 0.0 && dy <= 0.0 {
            // Inside
            if dx > dy {
                // Closer to vertical edge
                (vec2(nx, 0.0), dx)
            } else {
                // Closer to horizontal edge
                (vec2(0.0, ny), dy)
            }
        } else if dx <= 0.0 {
            // Outside but closer to horizontal edge
            (vec2(0.0, ny), dy)
        } else if dy <= 0.0 {
            // Outside but closer to vertical edge
            (vec2(nx, 0.0), dx)
        } else {
            // Outside but closer to vertex
            let normal = vec2(nx, ny);
            let normal = normal / normal.len();
            (normal, (dx * dx + dy * dy).sqrt())
        };

        if distance < circle_radius {
            Some((normal, circle_radius - distance))
        } else {
            None
        }
    }
    fn player_action(&mut self, player: &mut Player) {
        if let Some(action) = player.action.take() {
            match action {
                PlayerAction::MovingTo { pos, finish_action } => {
                    let finished = (player.pos - pos).len() <= player.interaction_range
                        && self.finish_action(player, finish_action)
                        || (player.pos - pos).len()
                            <= self.rules.player_movement_speed / self.ticks_per_second;
                    if !finished {
                        let dir = pos - player.pos;
                        let dir = dir / dir.len();
                        let new_pos = player.pos
                            + dir * self.rules.player_movement_speed / self.ticks_per_second;
                        player.pos = new_pos;
                        player.action = Some(PlayerAction::MovingTo { pos, finish_action });
                    }
                }
                PlayerAction::Crafting {
                    item_id,
                    recipe,
                    time_left,
                } => {
                    let time_left = time_left - 1.0 / self.ticks_per_second;
                    if time_left <= 0.0 {
                        let hand_item = &mut player.item;
                        let mut item = self.remove_item_id(item_id);
                        let (conditions, ingredient2) = match &item {
                            Some(item) => (
                                Some(
                                    self.get_tile(item.pos.map(|x| x as i64))
                                        .unwrap()
                                        .biome
                                        .clone(),
                                ),
                                Some(item.item_type.clone()),
                            ),
                            None => (None, None),
                        };
                        if recipe.ingredients_equal(hand_item.take(), ingredient2, conditions) {
                            *hand_item = recipe.result1;
                            if let Some(item) = item.take() {
                                if let Some(item_type) = recipe.result2 {
                                    self.spawn_item(item_type, item.pos);
                                }
                            }
                            self.play_sound(Sound::Craft, player.pos);
                        } else if let Some(item) = item {
                            self.spawn_item(item.item_type, item.pos);
                        }
                    } else {
                        player.action = Some(PlayerAction::Crafting {
                            item_id,
                            recipe,
                            time_left,
                        });
                    }
                }
            }
        }
    }
    fn finish_action(&mut self, player: &mut Player, finish_action: Option<MomentAction>) -> bool {
        if let Some(finish_action) = finish_action {
            match finish_action {
                MomentAction::Interact { id } => {
                    let ingredient1 = &mut player.item;
                    let (conditions, ingredient2) = match self.chunked_world.get_item(id) {
                        Some(item) => (
                            Some(
                                self.get_tile(item.pos.map(|x| x as i64))
                                    .unwrap()
                                    .biome
                                    .clone(),
                            ),
                            Some(item.item_type.clone()),
                        ),
                        None => (None, None),
                    };
                    let recipe = self.resource_pack.recipes.iter().find(|recipe| {
                        recipe.ingredients_equal(
                            ingredient1.clone(),
                            ingredient2.clone(),
                            conditions.clone(),
                        )
                    });
                    if let Some(recipe) = recipe {
                        player.action = Some(PlayerAction::Crafting {
                            item_id: id,
                            recipe: recipe.clone(),
                            time_left: recipe.craft_time,
                        });
                    }
                }
                MomentAction::Drop { pos } => {
                    let hand_item = &mut player.item;
                    if let Some(item_type) = hand_item.take() {
                        self.spawn_item(item_type, pos);
                        self.play_sound(Sound::PutDown, pos);
                    }
                }
                MomentAction::PickUp { id } => {
                    let hand_item = &mut player.item;
                    let mut item = self.chunked_world.remove_item(id);
                    let ground_item = match &item {
                        Some(item) => Some(item.item_type.clone()),
                        None => None,
                    };
                    if let None = hand_item {
                        if let Some(item_type) = ground_item {
                            if self.resource_pack.items[&item_type].pickable {
                                item.take();
                                *hand_item = Some(item_type);
                                self.play_sound(Sound::PickUp, player.pos);
                            }
                        }
                    }
                    if let Some(item) = item {
                        self.spawn_item(item.item_type, item.pos);
                    }
                }
            }
            true
        } else {
            false
        }
    }
}
