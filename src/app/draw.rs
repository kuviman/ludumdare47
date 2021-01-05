use super::*;

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
        let mut selected_item = None;
        let mut selected_player = None;
        if let Some(data) = self.players.get(&self.player_id) {
            self.draw_circle(framebuffer, data.pos, data.size, Color::GREEN);
        }
        if let Some(pos) = selected_pos {
            if let Some(item) = self.view.items.iter().find(|item| {
                (item.pos - pos).len() <= self.view.item_properties[&item.item_type].size
            }) {
                self.draw_circle(
                    framebuffer,
                    item.pos,
                    self.view.item_properties[&item.item_type].size,
                    Color::rgba(1.0, 1.0, 1.0, 0.5),
                );
                selected_item = Some(item);
            } else if let Some(player) = self
                .view
                .players
                .iter()
                .find(|e| (e.pos - pos).len() <= e.radius)
            {
                if let Some(data) = self.players.get(&player.id) {
                    if player.id != self.player_id {
                        self.draw_circle(
                            framebuffer,
                            data.pos,
                            player.radius,
                            Color::rgba(1.0, 1.0, 1.0, 0.5),
                        );
                    }
                }
                selected_player = Some(player);
            }
        }

        let mut instances: HashMap<model::ItemType, Vec<ez3d::Instance>> = HashMap::new();
        for item in &self.view.items {
            let pos = item.pos;
            let height = self.tile_mesh.get_height(pos).unwrap_or(0.0);
            let pos = pos.extend(height);
            instances
                .entry(item.item_type.clone())
                .or_default()
                .push(ez3d::Instance {
                    i_pos: pos,
                    i_size: 0.5,
                    i_rotation: 0.0,
                    i_color: Color::WHITE,
                });
        }
        for (item_type, instances) in instances {
            let obj = &self.resource_pack.items[&item_type].model;
            self.ez3d.draw(
                framebuffer,
                &self.camera,
                &self.light,
                obj.vb(),
                instances.into_iter(),
            );
        }
        for player in &self.view.players {
            let data = self
                .players
                .entry(player.id)
                .or_insert(PlayerData::new(player));
            let mut pos = data.pos.extend(data.step());
            let rotation = data.rotation;
            let height = self.tile_mesh.get_height(pos.xy()).unwrap_or(0.0);
            pos.z += height;
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
                    i_color: player.colors.skin,
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
                    i_color: player.colors.shirt,
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
                    i_color: player.colors.pants,
                }),
            );
            if let Some(item) = &player.item {
                self.ez3d.draw(
                    framebuffer,
                    &self.camera,
                    &self.light,
                    self.resource_pack.items[item].model.vb(),
                    std::iter::once(ez3d::Instance {
                        i_pos: pos + Vec2::rotated(vec2(0.45, 0.0), rotation).extend(0.6),
                        i_size: 0.5,
                        i_rotation: -rotation,
                        i_color: Color::WHITE,
                    }),
                );
            }
        }
        if let Some(item) = selected_item {
            let text = item.item_type.to_string();
            let pos = item.pos;
            let pos = pos.extend(self.tile_mesh.get_height(pos).unwrap_or(0.0));
            self.geng.default_font().draw_aligned(
                framebuffer,
                &text,
                self.camera.world_to_screen(self.framebuffer_size, pos) + vec2(0.0, 20.0),
                0.5,
                32.0,
                Color::WHITE,
            );
        } else if let Some(player) = selected_player {
            if let Some(data) = self.players.get(&player.id) {
                let mut text = if player.id == self.player_id {
                    "Me"
                } else {
                    "Player"
                }
                .to_owned();
                if let Some(item) = &player.item {
                    text = format!("{}, holding {}", text, item);
                }
                let pos = data.pos;
                let pos = pos.extend(self.tile_mesh.get_height(pos).unwrap());
                self.geng.default_font().draw_aligned(
                    framebuffer,
                    &text,
                    self.camera.world_to_screen(self.framebuffer_size, pos) + vec2(0.0, 20.0),
                    0.5,
                    32.0,
                    Color::WHITE,
                );
            }
        }

        if let Some(player) = self
            .view
            .players
            .iter()
            .find(|player| player.id == self.player_id)
        {
            let data = &self.players[&player.id];
            if let Some(action) = &player.action {
                match action {
                    model::PlayerAction::Crafting { time_left, .. } => {
                        let text = format!("{:.1}", time_left);
                        let pos = data.pos;
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
