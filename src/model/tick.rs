use super::*;

impl Model {
    pub fn tick(&mut self) {
        let mut timer = Timer::new();
        self.current_time += 1;
        if self.current_time >= self.day_length + self.night_length {
            self.current_time = 0;
        }
        let ids: Vec<Id> = self.entities.keys().copied().collect();
        for id in ids {
            let mut entity = self.entities.get(&id).unwrap().clone();
            if let Some(move_to) = entity.move_to {
                let dir_x = (move_to.0.x as i32 - entity.pos.x as i32).signum();
                let dir_y = (move_to.0.y as i32 - entity.pos.y as i32).signum();
                if move_to.1
                    && (entity.pos.x as i32 - move_to.0.x as i32).abs() <= 1
                    && (entity.pos.y as i32 - move_to.0.y as i32).abs() <= 1
                {
                    let ingredient1 = &mut entity.item;
                    let mut structure = self.structures.remove(&move_to.0);
                    let conditions = self.get_tile(move_to.0).unwrap().biome;
                    let ingredient2 = match &structure {
                        Some(structure) => Some(structure.structure_type),
                        None => None,
                    };
                    let recipe = self.recipes.iter().find(|recipe| {
                        recipe.ingredients_equal(*ingredient1, ingredient2, conditions)
                    });
                    if let Some(recipe) = recipe {
                        *ingredient1 = recipe.result1;
                        structure.take();
                        if let Some(structure_type) = recipe.result2 {
                            let structure = Structure {
                                pos: move_to.0,
                                structure_type,
                            };
                            self.structures.insert(structure.pos, structure);
                        }
                        self.play_sound(Sound::Craft, self.sound_distance, move_to.0);
                        entity.move_to = None;
                    } else if let Some(StructureType::Raft) = ingredient2 {
                        // entity.pos = move_to.0;
                        entity.controllable = false;
                        entity.move_to = Some((
                            vec2(
                                (entity.pos.x as i32 + dir_x) as usize,
                                (entity.pos.y as i32 + dir_y) as usize,
                            ),
                            false,
                        ));
                        self.structures.remove(&structure.take().unwrap().pos);
                    } else if let Some(StructureType::Item {
                        item: Item::TreasureMark,
                    }) = ingredient2
                    {
                        // Stop forward checks to prevent picking it up
                    } else if let Some(StructureType::Statue) = ingredient2 {
                        if let Some(item) = ingredient1.take() {
                            self.score += match self.scores_map.get(&item) {
                                Some(score) => *score,
                                None => 0,
                            };
                            self.play_sound(Sound::StatueGift, self.sound_distance, move_to.0);
                        }
                    } else if let Some(_) = ingredient1 {
                        if let None = ingredient2 {
                            structure.take();
                            let structure = Structure {
                                pos: move_to.0,
                                structure_type: StructureType::Item {
                                    item: ingredient1.take().unwrap(),
                                },
                            };
                            self.structures.insert(structure.pos, structure);
                            self.play_sound(Sound::PutDown, self.sound_distance, move_to.0);
                        }
                        entity.move_to = None;
                    } else if let Some(structure_type) = ingredient2 {
                        if let StructureType::Item { item } = structure_type {
                            structure.take();
                            *ingredient1 = Some(item);
                            self.play_sound(Sound::PickUp, self.sound_distance, move_to.0);
                        }
                        entity.move_to = None;
                    }
                    if let Some(structure) = structure {
                        self.structures.insert(structure.pos, structure);
                    }
                }
                if entity.pos == move_to.0 {
                    entity.move_to = None;
                } else if entity.move_to != None {
                    if let Some(mut new_pos) = self.pathfind(entity.pos, move_to.0) {
                        if let Some(tile) = self.get_tile(new_pos) {
                            if Biome::Water != tile.biome && self.is_traversable_tile(new_pos) {
                                entity.pos = new_pos;
                                entity.controllable = true;
                            } else if !entity.controllable {
                                if new_pos.x <= 0
                                    || new_pos.x >= self.size.x - 1
                                    || new_pos.y <= 0
                                    || new_pos.y >= self.size.y - 1
                                {
                                    new_pos.x = self.size.x - 1 - new_pos.x;
                                    new_pos.y = self.size.y - 1 - new_pos.y;
                                }
                                entity.pos = new_pos;
                                entity.move_to = Some((
                                    vec2(
                                        (entity.pos.x as i32 + dir_x) as usize,
                                        (entity.pos.y as i32 + dir_y) as usize,
                                    ),
                                    false,
                                ));
                            }
                        }
                    }
                }
            }
            entity.view_range =
                self.calc_view_range()
                    .max(if let Some(Item::Torch) = entity.item {
                        self.rules.torch_light
                    } else {
                        0.0
                    });
            *self.entities.get_mut(&id).unwrap() = entity;
        }

        println!("Moved in {:?}", timer.tick());

        let mut view = HashSet::new();
        for entity in self.entities.values() {
            let view_range = if let Some(Item::Torch) = entity.item {
                self.rules.torch_light.max(entity.view_range)
            } else {
                entity.view_range
            };
            Self::add_view_radius(&mut view, entity.pos, entity.view_range);
        }
        for light_source in self.structures.values().filter(|structure| {
            structure.structure_type == StructureType::Campfire
                || structure.structure_type == StructureType::Statue
                || structure.structure_type == StructureType::Item { item: Item::Torch }
        }) {
            Self::add_view_radius(
                &mut view,
                light_source.pos,
                match light_source.structure_type {
                    StructureType::Campfire => self.rules.campfire_light,
                    StructureType::Statue => self.rules.statue_light,
                    StructureType::Item { item: Item::Torch } => self.rules.torch_light,
                    _ => unreachable!(),
                },
            );
        }

        println!("Got view in {:?}", timer.tick());

        let mut extinguished_positions = Vec::new();
        for structure in self.structures.values() {
            match structure.structure_type {
                StructureType::Campfire | StructureType::Item { item: Item::Torch } => {
                    if global_rng().gen_range(0.0, 1.0) < self.rules.fire_extinguish_chance {
                        extinguished_positions.push(structure.pos);
                    }
                }
                _ => (),
            }
        }
        for pos in extinguished_positions {
            self.structures.remove(&pos);
        }
        println!("Extinguish in {:?}", timer.tick());

        for _ in
            0..(self.size.x as f32 * self.size.y as f32 * self.rules.regeneration_percent) as usize
        {
            let pos = vec2(
                global_rng().gen_range(0, self.size.x),
                global_rng().gen_range(0, self.size.y),
            );
            if !view.contains(&pos) {
                self.structures.remove(&pos);
                self.generate_tile(pos);
            }
        }
        println!("Regen in {:?}", timer.tick());
    }
}
