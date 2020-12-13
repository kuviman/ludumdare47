use super::*;

impl Model {
    pub fn tick(&mut self) {
        self.current_time += 1;
        if self.current_time >= self.day_length + self.night_length {
            self.current_time = 0;
        }
        let ids: Vec<Id> = self.entities.keys().copied().collect();
        for id in ids {
            let mut entity = self.entities.get(&id).unwrap().clone();
            self.entity_action(&mut entity);

            // Collide with items
            for item in self.items.values() {
                if !item.item_type.is_traversable() {
                    let dir = entity.pos - item.pos;
                    let distance = dir.len();
                    if distance <= entity.radius + item.size {
                        let penetration = entity.radius + item.size - distance;
                        let normal = dir / distance;
                        entity.pos += normal * penetration;
                    }
                }
            }

            // Collide with entities
            if let Some((normal, penetration)) = self.entities.values().find_map(|e| {
                if e.id == entity.id {
                    return None;
                }

                let dir = entity.pos - e.pos;
                let distance = dir.len();
                if distance <= entity.radius {
                    let penetration = entity.radius + e.radius - distance;
                    let normal = dir / distance;
                    Some((normal, penetration))
                } else {
                    None
                }
            }) {
                entity.pos += normal * penetration;
            }

            // Collide with tiles
            for x in (-entity.radius.ceil() as i64)..(entity.radius.ceil() as i64 + 1) {
                for y in (-entity.radius.ceil() as i64)..(entity.radius.ceil() as i64 + 1) {
                    let pos = vec2(x, y) + entity.pos.map(|x| x as i64);
                    if let Some((normal, penetration)) = match self.get_tile(pos) {
                        Some(tile) => match tile.biome {
                            Biome::Lake | Biome::Ocean => Self::collide(
                                entity.pos,
                                entity.radius,
                                tile.pos.map(|x| x as f32),
                                1.0,
                            ),
                            _ => None,
                        },
                        None => None,
                    } {
                        entity.pos += normal * penetration;
                    }
                }
            }

            entity.view_range =
                self.calc_view_range()
                    .max(if let Some(ItemType::Torch) = entity.item {
                        self.rules.torch_light
                    } else {
                        0.0
                    });
            *self.entities.get_mut(&id).unwrap() = entity;
        }

        let mut view = HashSet::new();
        for entity in self.entities.values() {
            Self::add_view_radius(&mut view, entity.pos, entity.view_range);
        }
        for light_source in self.items.values().filter(|item| {
            item.item_type == ItemType::Campfire
                || item.item_type == ItemType::Statue
                || item.item_type == ItemType::Torch
        }) {
            Self::add_view_radius(
                &mut view,
                light_source.pos.map(|x| x as f32),
                match light_source.item_type {
                    ItemType::Campfire => self.rules.campfire_light,
                    ItemType::Statue => self.rules.statue_light,
                    ItemType::Torch => self.rules.torch_light,
                    _ => unreachable!(),
                },
            );
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
    fn entity_action(&mut self, entity: &mut Entity) {
        if let Some(action) = entity.action.take() {
            match action {
                EntityAction::MovingTo { pos, finish_action } => {
                    let finished = (entity.pos - pos).len() <= entity.interaction_range
                        && self.finish_action(entity, finish_action)
                        || (entity.pos - pos).len()
                            <= self.rules.entity_movement_speed / self.ticks_per_second;
                    if !finished {
                        let dir = pos - entity.pos;
                        let dir = dir / dir.len();
                        let new_pos = entity.pos
                            + dir * self.rules.entity_movement_speed / self.ticks_per_second;
                        entity.pos = new_pos;
                        entity.action = Some(EntityAction::MovingTo { pos, finish_action });
                    }
                }
                EntityAction::Crafting {
                    item_id,
                    recipe,
                    time_left,
                } => {
                    let time_left = time_left - 1.0 / self.ticks_per_second;
                    if time_left <= 0.0 {
                        let hand_item = &mut entity.item;
                        let mut item = self.remove_item_id(item_id);
                        let (conditions, ingredient2) = match &item {
                            Some(item) => (
                                Some(self.get_tile(item.pos.map(|x| x as i64)).unwrap().biome),
                                Some(item.item_type),
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
                            self.play_sound(Sound::Craft, self.sound_distance, entity.pos);
                        } else if let Some(item) = item {
                            self.spawn_item(item.item_type, item.pos);
                        }
                    } else {
                        entity.action = Some(EntityAction::Crafting {
                            item_id,
                            recipe,
                            time_left,
                        });
                    }
                }
            }
        }
    }
    fn finish_action(&mut self, entity: &mut Entity, finish_action: Option<MomentAction>) -> bool {
        if let Some(finish_action) = finish_action {
            match finish_action {
                MomentAction::Interact { id } => {
                    let ingredient1 = &mut entity.item;
                    let (conditions, ingredient2) = match self.items.get(&id) {
                        Some(item) => (
                            Some(self.get_tile(item.pos.map(|x| x as i64)).unwrap().biome),
                            Some(item.item_type),
                        ),
                        None => (None, None),
                    };
                    let recipe = self.recipes.iter().find(|recipe| {
                        recipe.ingredients_equal(*ingredient1, ingredient2, conditions)
                    });
                    if let Some(recipe) = recipe {
                        entity.action = Some(EntityAction::Crafting {
                            item_id: id,
                            recipe: recipe.clone(),
                            time_left: recipe.craft_time,
                        });
                    } else if let Some(ItemType::Statue) = ingredient2 {
                        if let Some(item) = ingredient1.take() {
                            self.score += match self.scores_map.get(&item) {
                                Some(score) => *score,
                                None => 0,
                            };
                            self.play_sound(Sound::StatueGift, self.sound_distance, entity.pos);
                        }
                    }
                }
                MomentAction::Drop { pos } => {
                    let hand_item = &mut entity.item;
                    if let Some(item_type) = hand_item.take() {
                        self.spawn_item(item_type, pos);
                        self.play_sound(Sound::PutDown, self.sound_distance, pos);
                    }
                }
                MomentAction::PickUp { id } => {
                    let hand_item = &mut entity.item;
                    let mut item = self.items.remove(&id);
                    let ground_item = match &item {
                        Some(item) => Some(item.item_type),
                        None => None,
                    };
                    if let None = hand_item {
                        if let Some(item_type) = ground_item {
                            if item_type.is_pickable() {
                                item.take();
                                *hand_item = Some(item_type);
                                self.play_sound(Sound::PickUp, self.sound_distance, entity.pos);
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
