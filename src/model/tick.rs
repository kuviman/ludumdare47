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
        if entity.components.player.is_some() {
            self.player_action(&mut entity);
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

    fn player_action(&mut self, entity: &mut Entity) {
        if let Some(action) = entity.components.player.as_mut().unwrap().action.take() {
            match action {
                PlayerAction::MovingTo { pos, finish_action } => {
                    let entity_pos = entity.pos.unwrap();
                    let finished = (entity_pos - pos).len()
                        <= entity.components.player.as_ref().unwrap().interaction_range
                        && self.finish_action(entity, finish_action)
                        || (entity_pos - pos).len()
                            <= self.rules.player_movement_speed / self.ticks_per_second;
                    if !finished {
                        let player = entity.components.player.as_mut().unwrap();
                        let dir = pos - entity_pos;
                        let dir = dir / dir.len();
                        let new_pos = entity_pos
                            + dir * self.rules.player_movement_speed / self.ticks_per_second;
                        player.action = Some(PlayerAction::MovingTo { pos, finish_action });
                        entity.pos = Some(new_pos);
                    }
                }
                PlayerAction::Crafting {
                    item_id,
                    recipe,
                    time_left,
                } => {
                    let player = entity.components.player.as_mut().unwrap();
                    let time_left = time_left - 1.0 / self.ticks_per_second;
                    if time_left <= 0.0 {
                        let hand_item = &mut player.item;
                        let mut item = self.chunked_world.remove_entity(item_id);
                        let (conditions, ingredient2) = match &item {
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
                        };
                        if recipe.ingredients_equal(hand_item.take(), ingredient2, conditions) {
                            *hand_item = recipe.result1;
                            if let Some(item) = item.take() {
                                if let Some(item_type) = recipe.result2 {
                                    self.spawn_entity(item_type, item.pos.unwrap());
                                }
                            }
                            self.play_sound(Sound::Craft, entity.pos.unwrap());
                        } else if let Some(item) = item {
                            self.chunked_world.insert_entity(item).unwrap();
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
    fn finish_action(&mut self, entity: &mut Entity, finish_action: Option<MomentAction>) -> bool {
        let player = entity.components.player.as_mut().unwrap();
        if let Some(finish_action) = finish_action {
            match finish_action {
                MomentAction::Interact { id } => {
                    let ingredient1 = &mut player.item;
                    let (conditions, ingredient2) = match self.chunked_world.get_entity(id) {
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
                        self.spawn_entity(item_type, pos);
                        self.play_sound(Sound::PutDown, pos);
                    }
                }
                MomentAction::PickUp { id } => {
                    let hand_item = &mut player.item;
                    let mut ground_item = self.chunked_world.remove_entity(id);
                    if let None = hand_item {
                        if let Some(item_type) = &mut ground_item {
                            if let Some(_) = item_type.components.pickable {
                                *hand_item = Some(item_type.entity_type.clone());
                                ground_item.take();
                                self.play_sound(Sound::PickUp, entity.pos.unwrap());
                            }
                        }
                    }
                    if let Some(item) = ground_item {
                        self.chunked_world.insert_entity(item).unwrap();
                    }
                }
            }
            true
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
