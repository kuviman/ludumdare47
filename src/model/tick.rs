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
            if let Some(move_to) = entity.move_to {
                let dir = move_to.0 - entity.pos;
                let dir = dir / dir.len();
                if move_to.1 && (entity.center() - move_to.0).len() <= entity.size / 2.0 {
                    let ingredient1 = &mut entity.item;
                    let mut item = self.remove_item(move_to.0.map(|x| x as i64));
                    let conditions = self.tiles.get(&move_to.0.map(|x| x as i64)).unwrap().biome;
                    let ingredient2 = match &item {
                        Some(item) => Some(item.item_type),
                        None => None,
                    };
                    let recipe = self.recipes.iter().find(|recipe| {
                        recipe.ingredients_equal(*ingredient1, ingredient2, conditions)
                    });
                    if let Some(recipe) = recipe {
                        *ingredient1 = recipe.result1;
                        item.take();
                        if let Some(item_type) = recipe.result2 {
                            self.spawn_item(item_type, move_to.0);
                        }
                        self.play_sound(Sound::Craft, self.sound_distance, move_to.0);
                        entity.move_to = None;
                    } else if let Some(ItemType::Raft) = ingredient2 {
                        entity.controllable = false;
                        entity.move_to =
                            Some((vec2(entity.pos.x + dir.x, entity.pos.y + dir.y), false));
                        self.remove_item(item.take().unwrap().pos.map(|x| x as i64))
                            .unwrap();
                    } else if let Some(ItemType::TreasureMark) = ingredient2 {
                        // Stop forward checks to prevent picking it up
                    } else if let Some(ItemType::Statue) = ingredient2 {
                        if let Some(item) = ingredient1.take() {
                            self.score += match self.scores_map.get(&item) {
                                Some(score) => *score,
                                None => 0,
                            };
                            self.play_sound(Sound::StatueGift, self.sound_distance, move_to.0);
                        }
                    }
                    if let Some(item) = item {
                        self.spawn_item(item.item_type, item.pos);
                    }
                }
                if (entity.pos - move_to.0).len() <= 0.05 {
                    entity.move_to = None;
                } else if entity.move_to != None {
                    let new_pos =
                        entity.pos + dir * self.rules.entity_movement_speed / self.ticks_per_second;
                    let new_pos_int = new_pos.map(|x| x as i64);
                    if let Some(tile) = self.tiles.get(&new_pos_int) {
                        if Biome::Water != tile.biome {
                            entity.pos = new_pos;
                            entity.controllable = true;
                        } else if !entity.controllable {
                            entity.pos = new_pos;
                        }
                    }
                }
            }

            // Collide
            if let Some((normal, penetration)) = self.items.values().find_map(|item| {
                if !item.item_type.is_traversable() {
                    let dir = entity.center() - item.center();
                    let distance = dir.len();
                    return if distance <= entity.size / 2.0 {
                        let penetration = entity.size / 2.0 + item.size / 2.0 - distance;
                        let normal = dir / distance;
                        Some((normal, penetration))
                    } else {
                        None
                    };
                }
                None
            }) {
                entity.pos += normal * penetration;
            }
            if let Some((normal, penetration)) = self.entities.values().find_map(|e| {
                if e.id == entity.id {
                    return None;
                }

                let dir = entity.center() - e.center();
                let distance = dir.len();
                if distance <= entity.size / 2.0 {
                    let penetration = entity.size / 2.0 + e.size / 2.0 - distance;
                    let normal = dir / distance;
                    Some((normal, penetration))
                } else {
                    None
                }
            }) {
                entity.pos += normal * penetration;
            }

            // Round map
            if !entity.controllable {
                if entity.pos.x <= 0.0
                    || entity.pos.x >= self.size.x as f32 - 1.0
                    || entity.pos.y <= 0.0
                    || entity.pos.y >= self.size.y as f32 - 1.0
                {
                    entity.pos.x = self.size.x as f32 - 1.0 - entity.pos.x;
                    entity.pos.y = self.size.y as f32 - 1.0 - entity.pos.y;
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

        let mut extinguished_positions = Vec::new();
        for item in self.items.values() {
            match item.item_type {
                ItemType::Campfire | ItemType::Torch => {
                    if global_rng().gen_range(0.0, 1.0) < self.rules.fire_extinguish_chance {
                        extinguished_positions.push(item.pos.map(|x| x as i64));
                    }
                }
                _ => (),
            }
        }
        for pos in extinguished_positions {
            self.remove_item(pos).unwrap();
        }

        for _ in
            0..(self.size.x as f32 * self.size.y as f32 * self.rules.regeneration_percent) as usize
        {
            let pos = vec2(
                global_rng().gen_range(0, self.size.x as i64),
                global_rng().gen_range(0, self.size.y as i64),
            );
            if !view.contains(&pos) {
                self.remove_item(pos);
                self.generate_tile(pos);
            }
        }
    }
}
