use super::*;
use client_entity::CompRenderable;

impl App {
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        self.light = light::Uniforms::new(&self.view);

        ugli::clear(framebuffer, Some(Color::BLACK), Some(1.0));
        self.camera_controls.draw(&mut self.camera, framebuffer);

        self.tile_mesh.draw(framebuffer, &self.camera, &self.light);

        let selected_pos = self
            .tile_mesh
            .intersect(self.camera.pixel_ray(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ))
            .map(|pos| pos.xy());
        let mut selected_entity = None;
        if let Some(entity) = self.entities.get(&self.player_id) {
            let pos = if let Some(interpolate) = &entity.extra_components.interpolate {
                interpolate.current_pos
            } else {
                entity.pos.unwrap()
            };
            self.draw_circle(
                framebuffer,
                pos,
                entity.size.unwrap(),
                Color::GREEN,
            );
        }
        if let Some(pos) = selected_pos {
            if let Some(entity) = self.view.get_closest_entity(pos) {
                if entity.id != self.player_id {
                    let entity = self.entities.get(&entity.id).unwrap();
                    self.draw_circle(
                        framebuffer,
                        entity.pos.unwrap(),
                        entity.size.unwrap(),
                        Color::rgba(1.0, 1.0, 1.0, 0.5),
                    );
                }
                selected_entity = Some(entity.id);
            }
        }

        // Prepare entities' models
        let mut instances: HashMap<model::EntityType, Vec<ez3d::Instance>> = HashMap::new();
        for entity in self
            .entities
            .values()
            .filter(|entity| entity.extra_components.renderable.is_some())
        {
            let pos = if let Some(interpolate) = &entity.extra_components.interpolate {
                interpolate.current_pos
            } else {
                entity.pos.unwrap()
            };
            let height = self.tile_mesh.get_height(pos).unwrap_or(0.0);
            let pos = if let Some(_) = &entity.extra_components.hopping {
                pos.extend(entity.step() + height)
            } else {
                pos.extend(height)
            };
            let rotation = entity.extra_components.rotation.unwrap_or(0.0);
            match entity.extra_components.renderable.as_ref().unwrap() {
                CompRenderable::Simple => {
                    instances
                        .entry(entity.entity_type.clone())
                        .or_default()
                        .push(ez3d::Instance {
                            i_pos: pos,
                            i_size: 0.5,
                            i_rotation: -rotation,
                            i_color: Color::WHITE,
                        });
                }
                CompRenderable::Player => {
                    let colors = if let Some(model::CompController::Player { colors }) =
                        entity.controller.as_ref()
                    {
                        colors
                    } else {
                        unreachable!()
                    };
                    self.ez3d.draw(
                        framebuffer,
                        &self.camera,
                        &self.light,
                        self.assets.player.eyes.vb(),
                        std::iter::once(ez3d::Instance {
                            i_pos: pos,
                            i_size: 0.5,
                            i_rotation: -rotation,
                            i_color: Color::BLACK,
                        }),
                    );
                    self.ez3d.draw(
                        framebuffer,
                        &self.camera,
                        &self.light,
                        self.assets.player.skin.vb(),
                        std::iter::once(ez3d::Instance {
                            i_pos: pos,
                            i_size: 0.5,
                            i_rotation: -rotation,
                            i_color: colors.skin,
                        }),
                    );
                    self.ez3d.draw(
                        framebuffer,
                        &self.camera,
                        &self.light,
                        self.assets.player.shirt.vb(),
                        std::iter::once(ez3d::Instance {
                            i_pos: pos,
                            i_size: 0.5,
                            i_rotation: -rotation,
                            i_color: colors.shirt,
                        }),
                    );
                    self.ez3d.draw(
                        framebuffer,
                        &self.camera,
                        &self.light,
                        self.assets.player.pants.vb(),
                        std::iter::once(ez3d::Instance {
                            i_pos: pos,
                            i_size: 0.5,
                            i_rotation: -rotation,
                            i_color: colors.pants,
                        }),
                    );
                    if let Some(item) = &entity.holding.as_ref().unwrap().entity {
                        self.ez3d.draw(
                            framebuffer,
                            &self.camera,
                            &self.light,
                            self.resource_pack.entities[item].model.vb(),
                            std::iter::once(ez3d::Instance {
                                i_pos: pos + Vec2::rotated(vec2(0.45, 0.0), rotation).extend(0.6),
                                i_size: 0.5,
                                i_rotation: -rotation,
                                i_color: Color::WHITE,
                            }),
                        );
                    }
                }
            }
        }

        // Draw entities
        for (entity_type, instances) in instances {
            match self.resource_pack.entities.get(&entity_type) {
                Some(rendering) => {
                    self.ez3d.draw(
                        framebuffer,
                        &self.camera,
                        &self.light,
                        rendering.model.vb(),
                        instances.into_iter(),
                    );
                }
                None => (),
            }
        }

        if let Some(entity_id) = selected_entity {
            let entity = self.entities.get_mut(&entity_id).unwrap();
            let mut text;
            let pos;
            if let Some(CompRenderable::Player { .. }) = entity.extra_components.renderable {
                text = if entity.id == self.player_id {
                    "Me"
                } else {
                    "Player"
                }
                .to_owned();
                if let Some(item) = &entity.holding.as_ref().unwrap().entity {
                    text = format!("{}, holding {}", text, item);
                }
                pos = entity.pos.unwrap().extend(
                    self.tile_mesh
                        .get_height(entity.pos.unwrap())
                        .unwrap_or(0.0),
                );
            } else {
                text = entity.entity_type.to_string();
                pos = entity.pos.unwrap().extend(
                    self.tile_mesh
                        .get_height(entity.pos.unwrap())
                        .unwrap_or(0.0),
                );
            }
            self.geng.default_font().draw_aligned(
                framebuffer,
                &text,
                self.camera.world_to_screen(self.framebuffer_size, pos) + vec2(0.0, 20.0),
                0.5,
                32.0,
                Color::WHITE,
            );
        }

        if let Some(entity) = self.entities.get(&self.player_id) {
            if let Some(action) = &entity.action.as_ref().unwrap().current_action {
                match action {
                    model::EntityAction::Crafting { time_left, .. } => {
                        let text = format!("{:.1}", time_left);
                        let pos = entity.pos.unwrap();
                        let pos = pos.extend(self.tile_mesh.get_height(pos).unwrap());
                        self.geng.default_font().draw_aligned(
                            framebuffer,
                            &text,
                            self.camera.world_to_screen(self.framebuffer_size, pos)
                                + vec2(0.0, 50.0),
                            0.5,
                            32.0,
                            Color::WHITE,
                        );
                    }
                    _ => (),
                }
            }
        }
        self.geng.default_font().draw_aligned(
            framebuffer,
            &format!("Players online: {}", self.view.players_online),
            vec2(
                self.framebuffer_size.x as f32 / 2.0,
                self.framebuffer_size.y as f32 - 100.0,
            ),
            0.5,
            32.0,
            Color::WHITE,
        );
        self.geng.default_font().draw(
            framebuffer,
            self.traffic_counter.text(),
            vec2(32.0, 32.0),
            24.0,
            Color::WHITE,
        );
        self.ui_controller
            .draw(&mut self.ui_state.ui(), framebuffer);
    }
}
