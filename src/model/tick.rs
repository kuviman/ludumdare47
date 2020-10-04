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
                let dir_x = (move_to.0.x as i32 - entity.pos.x as i32).signum();
                let dir_y = (move_to.0.y as i32 - entity.pos.y as i32).signum();
                if move_to.1
                    && (entity.pos.x as i32 - move_to.0.x as i32).abs() <= 1
                    && (entity.pos.y as i32 - move_to.0.y as i32).abs() <= 1
                {
                    let ingredient1 = &mut entity.item;
                    let structure = self.get_structure(move_to.0);
                    let conditions = self.get_tile(move_to.0).unwrap().ground_type;
                    let ingredient2 = match structure {
                        Some((_, structure)) => Some(structure.structure_type),
                        None => None,
                    };
                    let recipe = self.recipes.iter().find(|recipe| {
                        recipe.ingredients_equal(*ingredient1, ingredient2, conditions)
                    });
                    if let Some(recipe) = recipe {
                        *ingredient1 = recipe.result1;
                        if let Some(structure_type) = recipe.result2 {
                            if let Some((structure_index, _)) = structure {
                                let structure = self.structures.get_mut(structure_index).unwrap();
                                structure.structure_type = structure_type;
                            } else {
                                self.structures.push(Structure {
                                    pos: move_to.0,
                                    structure_type: structure_type,
                                })
                            }
                        } else if let Some((structure_index, _)) = structure {
                            self.structures.remove(structure_index);
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
                        let structure_index = structure.unwrap().0;
                        self.structures.remove(structure_index);
                    } else if let Some(_) = ingredient1 {
                        if let None = ingredient2 {
                            self.structures.push(Structure {
                                pos: move_to.0,
                                structure_type: StructureType::Item {
                                    item: ingredient1.take().unwrap(),
                                },
                            });
                        }
                        entity.move_to = None;
                    } else if let Some(structure_type) = ingredient2 {
                        if let StructureType::Item { item } = structure_type {
                            let index = structure.unwrap().0;
                            self.structures.remove(index);
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

        let mut view = HashSet::new();
        for entity in self.entities.values() {
            Self::add_view_radius(&mut view, entity.pos, entity.view_range);
        }
        for light_source in self.structures.iter().filter(|structure| {
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

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = vec2(x, y);
                let structure = self.get_structure(pos);
                if let Some(structure) = structure {
                    match structure.1.structure_type {
                        StructureType::Campfire | StructureType::Item { item: Item::Torch } => {
                            if global_rng().gen_range(0.0, 1.0) < self.rules.fire_extinguish_chance
                            {
                                self.remove_at(pos);
                            }
                        }
                        _ => (),
                    }
                }
                if global_rng().gen_range(0.0, 1.0) < self.rules.regeneration_percent {
                    if !view.contains(&pos) {
                        self.remove_at(pos);
                        self.generate_tile(pos);
                    }
                }
            }
        }
    }
}
