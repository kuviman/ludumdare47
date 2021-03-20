use super::*;

impl Model {
    pub fn tick(&mut self) {
        let ids: Vec<Id> = self.chunked_world.entities().map(|e| e.id).collect();
        for id in ids {
            if let Some(entity) = self.chunked_world.get_entity(id) {
                let entity = entity.clone();
                self.update_entity(entity);
            }
        }
    }

    fn update_entity(&mut self, mut entity: Entity) {
        let entity_id = entity.id;
        if entity.action.is_some() {
            self.entity_action(&mut entity);
        }

        // Collide with entities
        if entity.components.collidable.is_some() {
            for other in self
                .chunked_world
                .entities_mut()
                .filter(|other| other.id != entity_id && other.components.collidable.is_some())
            {
                Self::check_entity_collision(&mut entity, other);
            }
        }

        // Collide with tiles
        if let Some(size) = entity.size {
            for x in (-size.ceil() as i64)..(size.ceil() as i64 + 1) {
                for y in (-size.ceil() as i64)..(size.ceil() as i64 + 1) {
                    let tile_pos = get_tile_pos(vec2(x as f32, y as f32) + entity.pos.unwrap());
                    self.check_entity_tile_collision(&mut entity, tile_pos);
                }
            }
        }

        *self.chunked_world.get_entity_mut(entity_id).unwrap() = entity;
        self.chunked_world.update_entity(entity_id);
    }

    fn check_entity_collision(entity: &mut Entity, other: &mut Entity) {
        match entities_collision(
            entity.pos.unwrap(),
            entity.size.unwrap(),
            other.pos.unwrap(),
            other.size.unwrap(),
        ) {
            CollisionResult::Collision {
                penetration,
                collision_normal,
            } => match entity.collidable.as_ref().unwrap().collision_type {
                CollisionType::Static => {
                    *other.pos.as_mut().unwrap() += -collision_normal * penetration;
                }
                CollisionType::Pushable => {
                    match other.collidable.as_ref().unwrap().collision_type {
                        CollisionType::Static => {
                            *entity.pos.as_mut().unwrap() += collision_normal * penetration;
                        }
                        CollisionType::Pushable => {
                            *entity.pos.as_mut().unwrap() += collision_normal * penetration / 2.0;
                            *other.pos.as_mut().unwrap() += collision_normal * penetration / 2.0;
                        }
                    }
                }
            },
            _ => (),
        }
    }

    fn check_entity_tile_collision(&mut self, entity: &mut Entity, tile_pos: Vec2<i64>) {
        match self.chunked_world.get_tile(tile_pos) {
            Some(tile) => {
                if self.resource_pack.biome_properties[&tile.biome].collidable {
                    match entity_tile_collision(
                        entity.pos.unwrap(),
                        entity.size.unwrap(),
                        tile_pos,
                        1.0,
                    ) {
                        CollisionResult::Collision {
                            penetration,
                            collision_normal,
                        } => {
                            *entity.pos.as_mut().unwrap() += collision_normal * penetration;
                        }
                        _ => (),
                    }
                }
            }
            None => (),
        }
    }

    fn entity_action(&mut self, entity: &mut Entity) {
        if let Some(action) = entity.action.as_mut().unwrap().current_action.take() {
            match action {
                EntityAction::MovingTo { target } => {
                    if let Some((target_pos, target_size)) = self.get_target(&target) {
                        let entity_pos = entity.pos.unwrap();
                        let distance = (entity_pos - target_pos).len();
                        if distance <= target_size
                            || distance <= self.rules.player_movement_speed / self.ticks_per_second
                        {
                            let entity_action = entity.action.as_mut().unwrap();
                            entity_action.current_action = entity_action.next_action.take();
                        } else {
                            entity.move_towards(
                                target_pos,
                                self.rules.player_movement_speed,
                                1.0 / self.ticks_per_second,
                            );
                            let entity_action = entity.action.as_mut().unwrap();
                            entity_action.current_action = Some(EntityAction::MovingTo { target });
                        }
                    }
                }
                EntityAction::Crafting {
                    target,
                    recipe,
                    time_left,
                } => {
                    let time_left = time_left - 1.0 / self.ticks_per_second;
                    if time_left <= 0.0 {
                        let hand_entity = &mut entity.holding.as_mut().unwrap().entity;
                        let (conditions, mut ground_entity) = match target {
                            ActionTarget::Entity { id } => {
                                let entity = self.chunked_world.remove_entity(id);
                                match entity {
                                    Some(entity) => (
                                        Some(
                                            self.chunked_world
                                                .get_tile(get_tile_pos(entity.pos.unwrap()))
                                                .unwrap()
                                                .biome
                                                .clone(),
                                        ),
                                        Some(entity),
                                    ),
                                    None => (None, None),
                                }
                            }
                            ActionTarget::Position { pos, .. } => (
                                Some(
                                    self.chunked_world
                                        .get_tile(get_tile_pos(pos))
                                        .unwrap()
                                        .biome
                                        .clone(),
                                ),
                                None,
                            ),
                        };
                        let ingredient2 = ground_entity.as_ref().map(|e| e.entity_type.clone());
                        if recipe.ingredients_equal(hand_entity.take(), ingredient2, conditions) {
                            *hand_entity = recipe.result1;
                            if let Some(entity) = ground_entity.take() {
                                if let Some(entity_type) = recipe.result2 {
                                    self.spawn_entity(entity_type, entity.pos.unwrap());
                                }
                            }
                            self.play_sound(Sound::Craft, entity.pos.unwrap());
                        } else if let Some(entity) = ground_entity {
                            self.chunked_world.insert_entity(entity).unwrap();
                        }
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = entity_action.next_action.take();
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::Crafting {
                            target,
                            recipe,
                            time_left,
                        });
                    }
                }
                EntityAction::Interact { target } => {
                    if self.can_interact(entity, &target) {
                        let ingredient1 = &mut entity.holding.as_mut().unwrap().entity;
                        let (conditions, ingredient2) = match target {
                            ActionTarget::Entity { id } => {
                                match self.chunked_world.get_entity(id) {
                                    Some(item) => (
                                        Some(
                                            self.chunked_world
                                                .get_tile(get_tile_pos(item.pos.unwrap()))
                                                .unwrap()
                                                .biome
                                                .clone(),
                                        ),
                                        Some(item.entity_type.clone()),
                                    ),
                                    None => (None, None),
                                }
                            }
                            ActionTarget::Position { pos, .. } => (
                                Some(
                                    self.chunked_world
                                        .get_tile(get_tile_pos(pos))
                                        .unwrap()
                                        .biome
                                        .clone(),
                                ),
                                None,
                            ),
                        };
                        let recipe = self.resource_pack.recipes.iter().find(|recipe| {
                            recipe.ingredients_equal(
                                ingredient1.clone(),
                                ingredient2.clone(),
                                conditions.clone(),
                            )
                        });
                        if let Some(recipe) = recipe {
                            let entity_action = entity.action.as_mut().unwrap();
                            entity_action.current_action = Some(EntityAction::Crafting {
                                target,
                                recipe: recipe.clone(),
                                time_left: recipe.craft_time,
                            });
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo {
                            target: target.clone(),
                        });
                        entity_action.next_action = Some(EntityAction::Interact { target });
                    }
                }
                EntityAction::Drop { pos } => {
                    let target = ActionTarget::Position {
                        pos,
                        target_size: entity.interaction.as_ref().unwrap().interaction_range,
                    };
                    if self.can_interact(entity, &target) {
                        let hand_item = &mut entity.holding.as_mut().unwrap().entity;
                        if let Some(item_type) = hand_item.take() {
                            self.spawn_entity(item_type, pos);
                            self.play_sound(Sound::PutDown, pos);
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo { target });
                        entity_action.next_action = Some(EntityAction::Drop { pos });
                    }
                }
                EntityAction::PickUp { id } => {
                    let target = ActionTarget::Entity { id };
                    if self.can_interact(entity, &target) {
                        let hand_item = &mut entity.holding.as_mut().unwrap().entity;
                        let mut ground_entity = self.chunked_world.remove_entity(id);
                        if let None = hand_item {
                            if let Some(e) = &mut ground_entity {
                                if e.pickable.is_some() {
                                    *hand_item = Some(e.entity_type.clone());
                                    ground_entity.take();
                                    self.play_sound(Sound::PickUp, entity.pos.unwrap());
                                }
                            }
                        }
                        if let Some(item) = ground_entity {
                            self.chunked_world.insert_entity(item).unwrap();
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo { target });
                        entity_action.next_action = Some(EntityAction::PickUp { id });
                    }
                }
            }
        }
    }

    fn get_target(&self, target: &ActionTarget) -> Option<(Vec2<f32>, f32)> {
        match target {
            ActionTarget::Position { pos, target_size } => Some((*pos, *target_size)),
            ActionTarget::Entity { id } => match self.chunked_world.get_entity(*id) {
                Some(target_entity) => {
                    Some((target_entity.pos.unwrap(), target_entity.size.unwrap()))
                }
                None => None,
            },
        }
    }

    fn can_interact(&self, entity: &Entity, target: &ActionTarget) -> bool {
        if let Some((target_pos, target_size)) = self.get_target(target) {
            let distance = (entity.pos.unwrap() - target_pos).len();
            distance
                <= entity.size.unwrap()
                    + target_size
                    + entity.interaction.as_ref().unwrap().interaction_range
        } else {
            false
        }
    }

    pub fn spawn_entity(&mut self, entity_type: EntityType, pos: Vec2<f32>) {
        let mut components = self.resource_pack.entity_components[&entity_type].clone();
        components.pos = Some(pos);
        self.chunked_world
            .insert_entity(Entity::new(
                &entity_type,
                components,
                self.id_generator.gen(),
            ))
            .unwrap();
    }
}
