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
        self.entity_action(&mut entity);

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

        if let Some(e) = self.chunked_world.get_entity_mut(entity_id) {
            *e = entity;
        }
        self.chunked_world
            .update_entity(entity_id, &mut self.id_generator);
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
                            *other.pos.as_mut().unwrap() += -collision_normal * penetration / 2.0;
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
        if entity.controller.is_some() {
            self.entity_action_decide(entity);
        }
        if entity.action.is_some() {
            self.entity_action_perform(entity);
        }
    }

    fn entity_action_decide(&self, entity: &mut Entity) {
        match entity.controller.as_ref().unwrap() {
            CompController::Player { .. } => (),
            CompController::BiomeRandomWalker { biome } => {
                let entity_action = entity.action.as_ref().unwrap();
                if entity_action.current_action.is_none() {
                    let mut random = global_rng();
                    let random_pos = entity.pos.unwrap()
                        + vec2(
                            random.gen_range(-10.0..=10.0),
                            random.gen_range(-10.0..=10.0),
                        );
                    if let Some(tile) = self.chunked_world.get_tile(get_tile_pos(random_pos)) {
                        if tile.biome == *biome {
                            let target = ActionTarget {
                                interaction_type: InteractionType::None,
                                target_type: TargetType::Position { pos: random_pos },
                            };
                            let entity_action = entity.action.as_mut().unwrap();
                            entity_action.current_action = Some(EntityAction::MovingTo { target });
                        }
                    }
                }
            }
        }
    }

    fn entity_action_perform(&mut self, entity: &mut Entity) {
        if let Some(action) = entity.action.as_mut().unwrap().current_action.take() {
            match action {
                EntityAction::MovingTo { target } => {
                    let (target_pos, reached) = self.reached_target(entity, &target);
                    if reached {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = entity_action.next_action.take();
                    } else {
                        entity.move_towards(
                            target_pos,
                            entity.movement_speed.unwrap(),
                            1.0 / self.ticks_per_second,
                        );
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo { target });
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
                        let (conditions, ingredient2) =
                            self.get_target_conditions(&target.target_type);
                        if recipe.ingredients_equal(
                            hand_entity.take().map(|e| e.entity_type),
                            ingredient2,
                            conditions,
                        ) {
                            *hand_entity = match &recipe.result1 {
                                Some(entity_type) => Some(Box::new(Entity::new(
                                    self.id_generator.gen(),
                                    entity_type,
                                    None,
                                    &self.resource_pack.entity_components,
                                ))),
                                None => None,
                            };
                            let target_pos = match &target.target_type {
                                TargetType::Entity { id } => {
                                    let entity = self.chunked_world.remove_entity(*id);
                                    entity.map(|e| e.pos.unwrap())
                                }
                                TargetType::Position { pos } => Some(*pos),
                            };
                            if let Some(target_pos) = target_pos {
                                if let Some(entity_type) = recipe.result2 {
                                    self.spawn_entity(&entity_type, target_pos);
                                }
                            }
                            self.play_sound(Sound::Craft, entity.pos.unwrap());
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
                EntityAction::Attacking {
                    target_entity_id,
                    time_left,
                } => {
                    let time_left = time_left - 1.0 / self.ticks_per_second;
                    if time_left <= 0.0 {
                        let weapon = entity.holding.as_ref().unwrap().entity.as_ref().unwrap();
                        if let Some(weapon) = &weapon.weapon {
                            self.damage_entity(target_entity_id, weapon);
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::Attacking {
                            target_entity_id,
                            time_left,
                        });
                    }
                }
                EntityAction::Interact { target } => {
                    let (_, reached) = self.reached_target(entity, &target);
                    if reached {
                        match &target.interaction_type {
                            InteractionType::Interact => {
                                let ingredient1 = entity
                                    .holding
                                    .as_ref()
                                    .unwrap()
                                    .entity
                                    .as_ref()
                                    .map(|e| e.entity_type.clone());
                                let (conditions, ingredient2) =
                                    self.get_target_conditions(&target.target_type);
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
                            }
                            InteractionType::Attack => {
                                if let TargetType::Entity { id } = &target.target_type {
                                    if let Some(holding) = &entity.holding {
                                        if let Some(holding) = &holding.entity {
                                            if let Some(weapon) = &holding.weapon {
                                                let time_left = weapon.attack_time;
                                                let entity_action = entity.action.as_mut().unwrap();
                                                entity_action.current_action =
                                                    Some(EntityAction::Attacking {
                                                        target_entity_id: *id,
                                                        time_left,
                                                    });
                                            }
                                        }
                                    }
                                }
                            }
                            InteractionType::None => (),
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo {
                            target: target.clone(),
                        });
                        entity_action.next_action = Some(EntityAction::Interact { target });
                    }
                }
                EntityAction::Use => {
                    self.entity_use(entity);
                }
                EntityAction::Drop { pos } => {
                    let target = ActionTarget {
                        interaction_type: InteractionType::Interact,
                        target_type: TargetType::Position { pos },
                    };
                    let (_, reached) = self.reached_target(entity, &target);
                    if reached {
                        let hand_entity = &mut entity.holding.as_mut().unwrap().entity;
                        if let Some(mut hand_entity) = hand_entity.take() {
                            hand_entity.pos = Some(pos);
                            self.chunked_world
                                .insert_entity(*hand_entity, &mut self.id_generator)
                                .unwrap();
                            self.play_sound(Sound::PutDown, pos);
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo { target });
                        entity_action.next_action = Some(EntityAction::Drop { pos });
                    }
                }
                EntityAction::PickUp { id } => {
                    let target = ActionTarget {
                        interaction_type: InteractionType::Interact,
                        target_type: TargetType::Entity { id },
                    };
                    let (_, reached) = self.reached_target(entity, &target);
                    if reached {
                        let hand_item = &mut entity.holding.as_mut().unwrap().entity;
                        let mut ground_entity = self.chunked_world.remove_entity(id);
                        if let None = hand_item {
                            if let Some(e) = &ground_entity {
                                if e.pickable.is_some() {
                                    *hand_item = Some(Box::new(ground_entity.take().unwrap()));
                                    self.play_sound(Sound::PickUp, entity.pos.unwrap());
                                }
                            }
                        }
                        if let Some(item) = ground_entity {
                            self.chunked_world
                                .insert_entity(item, &mut self.id_generator)
                                .unwrap();
                        }
                    } else {
                        let entity_action = entity.action.as_mut().unwrap();
                        entity_action.current_action = Some(EntityAction::MovingTo { target });
                        entity_action.next_action = Some(EntityAction::PickUp { id });
                    }
                }
            }
        } else {
            let entity_action = entity.action.as_mut().unwrap();
            entity_action.current_action = entity_action.next_action.take();
        }
    }

    fn entity_use(&mut self, entity: &mut Entity) {
        if let Some(hand_entity) = &entity.holding.as_ref().unwrap().entity {
            if let Some(usable) = &hand_entity.usable {
                let consumable = usable.consumable;
                match &usable.effect {
                    Effect::Spawn { entity_type } => {
                        self.spawn_entity(entity_type, entity.pos.unwrap());
                    }
                    Effect::Heal { heal_points } => {
                        let hp = *heal_points;
                        self.entity_change_hp(entity, hp);
                    }
                }
                if consumable {
                    entity.holding.as_mut().unwrap().entity.take();
                }
            }
        }
    }

    fn entity_change_hp(&mut self, entity: &mut Entity, heal_points: f32) {
        if let Some(hp) = &mut entity.hp {
            hp.current_hp = (hp.current_hp + heal_points).clamp(0.0, hp.max_hp);
            if hp.current_hp <= 0.0 {
                self.kill_entity(entity.id);
            }
        }
    }

    fn get_extra_range(entity: &Entity, interaction_type: &InteractionType) -> f32 {
        match interaction_type {
            InteractionType::None => 0.0,
            InteractionType::Interact => {
                entity.size.unwrap() + entity.interaction.as_ref().unwrap().interaction_range
            }
            InteractionType::Attack => {
                entity.size.unwrap()
                    + match &entity.holding.as_ref().unwrap().entity {
                        Some(weapon_entity) => {
                            if let Some(weapon) = &weapon_entity.weapon {
                                weapon.attack_distance
                            } else {
                                0.0
                            }
                        }
                        None => 0.0,
                    }
            }
        }
    }

    fn reached_target(&self, entity: &Entity, target: &ActionTarget) -> (Vec2<f32>, bool) {
        if let Some((target_pos, target_size)) = self.get_target(&target.target_type) {
            let entity_pos = entity.pos.unwrap();
            let distance = (entity_pos - target_pos).len();
            let extra_range = Self::get_extra_range(entity, &target.interaction_type);
            (
                target_pos,
                distance <= target_size + extra_range
                    || distance <= entity.movement_speed.unwrap() / self.ticks_per_second,
            )
        } else {
            (entity.pos.unwrap(), true)
        }
    }

    fn get_target(&self, target_type: &TargetType) -> Option<(Vec2<f32>, f32)> {
        match target_type {
            TargetType::Position { pos } => Some((*pos, 0.0)),
            TargetType::Entity { id } => match self.chunked_world.get_entity(*id) {
                Some(target_entity) => {
                    Some((target_entity.pos.unwrap(), target_entity.size.unwrap()))
                }
                None => None,
            },
        }
    }

    fn get_target_conditions(
        &mut self,
        target_type: &TargetType,
    ) -> (Option<Biome>, Option<EntityType>) {
        match target_type {
            TargetType::Entity { id } => match self.chunked_world.get_entity(*id) {
                Some(entity) => (
                    Some(
                        self.chunked_world
                            .get_tile(get_tile_pos(entity.pos.unwrap()))
                            .unwrap()
                            .biome
                            .clone(),
                    ),
                    Some(entity.entity_type.clone()),
                ),
                None => (None, None),
            },
            TargetType::Position { pos } => (
                Some(
                    self.chunked_world
                        .get_tile(get_tile_pos(*pos))
                        .unwrap()
                        .biome
                        .clone(),
                ),
                None,
            ),
        }
    }

    pub fn spawn_entity(&mut self, entity_type: &EntityType, pos: Vec2<f32>) {
        let entity = Entity::new(
            self.id_generator.gen(),
            entity_type,
            Some(pos),
            &self.resource_pack.entity_components,
        );
        self.chunked_world
            .insert_entity(entity, &mut self.id_generator)
            .unwrap();
    }

    fn damage_entity(&mut self, entity_id: Id, weapon: &CompWeapon) {
        if let Some(entity) = self.chunked_world.get_entity_mut(entity_id) {
            if let Some(hp) = entity.hp.as_mut() {
                hp.current_hp -= weapon.damage;
                if hp.current_hp <= 0.0 {
                    self.kill_entity(entity_id);
                }
            }
        }
    }

    fn kill_entity(&mut self, entity_id: Id) {
        self.chunked_world.remove_entity(entity_id);
    }
}
