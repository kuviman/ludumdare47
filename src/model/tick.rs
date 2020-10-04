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
                    let mut structure = self.structures.get_mut(&move_to.0).cloned();
                    let conditions = self.get_tile(move_to.0).unwrap().ground_type;
                    let ingredient2 = match &structure {
                        Some(structure) => Some(structure.structure_type),
                        None => None,
                    };
                    let recipe = self.recipes.iter().find(|recipe| {
                        recipe.ingredients_equal(*ingredient1, ingredient2, conditions)
                    });
                    if let Some(recipe) = recipe {
                        *ingredient1 = recipe.result1;
                        if let Some(structure_type) = recipe.result2 {
                            if let Some(structure) = &mut structure {
                                structure.structure_type = structure_type;
                            } else {
                                let structure = Structure {
                                    pos: move_to.0,
                                    structure_type: structure_type,
                                };
                                self.structures.insert(structure.pos, structure);
                            }
                        } else if let Some(structure) = &structure {
                            self.structures.remove(&structure.pos);
                        }
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
                        self.structures.remove(&structure.unwrap().pos);
                    } else if let Some(_) = ingredient1 {
                        if let None = ingredient2 {
                            let structure = Structure {
                                pos: move_to.0,
                                structure_type: StructureType::Item {
                                    item: ingredient1.take().unwrap(),
                                },
                            };
                            self.structures.insert(structure.pos, structure);
                        }
                        entity.move_to = None;
                    } else if let Some(structure_type) = ingredient2 {
                        if let StructureType::Item { item } = structure_type {
                            self.structures.remove(&structure.as_ref().unwrap().pos);
                            *ingredient1 = Some(item);
                        }
                        entity.move_to = None;
                    }
                }
                if entity.pos == move_to.0 {
                    entity.move_to = None;
                } else if entity.move_to != None {
                    let mut new_pos = vec2(
                        (entity.pos.x as i32 + dir_x) as usize,
                        (entity.pos.y as i32 + dir_y) as usize,
                    );
                    if let Some(tile) = self.get_tile(new_pos) {
                        if GroundType::Water != tile.ground_type
                            && self.is_traversable_tile(new_pos)
                        {
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
            Self::add_view_radius(&mut view, entity.pos, entity.view_range);
        }
        for light_source in self.structures.values().filter(|structure| {
            structure.structure_type == StructureType::Campfire
                || structure.structure_type == StructureType::Item { item: Item::Torch }
        }) {
            Self::add_view_radius(
                &mut view,
                light_source.pos,
                match light_source.structure_type {
                    StructureType::Campfire => self.rules.campfire_light,
                    StructureType::Item { item: Item::Torch } => self.rules.torch_light,
                    _ => unreachable!(),
                },
            );
        }

        println!("Got view in {:?}", timer.tick());

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = vec2(x, y);
                let structure = self.structures.get(&pos);
                if let Some(structure) = structure {
                    match structure.structure_type {
                        StructureType::Campfire | StructureType::Item { item: Item::Torch } => {
                            if global_rng().gen_range(0.0, 1.0) < self.rules.fire_extinguish_chance
                            {
                                self.structures.remove(&pos);
                            }
                        }
                        _ => (),
                    }
                }
                if global_rng().gen_range(0.0, 1.0) < self.rules.regeneration_percent {
                    if !view.contains(&pos) {
                        self.structures.remove(&pos);
                        self.generate_tile(pos);
                    }
                }
            }
        }

        println!("Regen in {:?}", timer.tick());
    }
}
