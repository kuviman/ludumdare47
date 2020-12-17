use super::*;

use noise::NoiseFn;

mod camera;
mod ez3d;
mod light;
mod resource_pack;
mod tile_mesh;
mod traffic;

use camera::Camera;
use ez3d::Ez3D;
pub use resource_pack::ResourcePack;
use tile_mesh::TileMesh;

#[derive(geng::Assets)]
pub struct PlayerAssets {
    eyes: ez3d::Obj,
    skin: ez3d::Obj,
    shirt: ez3d::Obj,
    pants: ez3d::Obj,
}

#[derive(geng::Assets)]
pub struct Assets {
    #[asset(path = "music.ogg")]
    music: geng::Sound,
    player: PlayerAssets,
    craft: geng::Sound,
    pickup: geng::Sound,
    walk: geng::Sound,
    blessing: geng::Sound,
    hi: geng::Sound,
    hello: geng::Sound,
    heyo: geng::Sound,
}

struct PlayerData {
    pos: Vec2<f32>,
    size: f32,
    target_pos: Vec2<f32>,
    speed: f32,
    rotation: f32,
    ampl: f32,
    t: f32,
}

impl PlayerData {
    fn new(player: &model::Player) -> Self {
        Self {
            pos: player.pos,
            size: player.radius,
            speed: 0.0,
            rotation: 0.0,
            target_pos: player.pos,
            ampl: 0.0,
            t: 0.0,
        }
    }
    fn step(&self) -> f32 {
        self.ampl * self.t.sin().abs() * 0.1
    }
    fn update(&mut self, player: &model::Player, delta_time: f32, view: &model::PlayerView) {
        self.size = player.radius;
        self.t += delta_time * 10.0;
        if player.pos != self.target_pos {
            self.target_pos = player.pos;
            self.speed = (player.pos - self.pos).len() / (2.0 / view.ticks_per_second);
        }
        let dpos = player.pos - self.pos;
        self.pos += dpos.clamp(self.speed * delta_time);
        if dpos.len() > 1e-9 {
            self.rotation = dpos.arg();
            self.ampl = (self.ampl + delta_time * 20.0).min(1.0);
        } else {
            self.ampl = (self.ampl - delta_time * 20.0).max(0.0);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Settings {
    volume: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { volume: 0.5 }
    }
}

struct UiState {
    geng: Rc<Geng>,
    settings: AutoSave<Settings>,
    volume_slider: geng::ui::Slider,
}

impl UiState {
    fn new(geng: &Rc<Geng>) -> Self {
        let ui_theme = Rc::new(geng::ui::Theme::default(geng));
        Self {
            geng: geng.clone(),
            settings: AutoSave::load(".settings"),
            volume_slider: geng::ui::Slider::new(&ui_theme),
        }
    }
    fn volume(&self) -> f64 {
        return self.settings.volume * 0.2;
    }
    fn ui<'a>(&'a mut self) -> impl geng::ui::Widget + 'a {
        use geng::ui;
        use geng::ui::*;
        let settings = &mut self.settings;
        let current_volume = settings.volume;
        ui::row![
            geng::ui::Text::new("volume", self.geng.default_font(), 24.0, Color::WHITE)
                .padding_right(24.0),
            self.volume_slider
                .ui(
                    current_volume,
                    0.0..=1.0,
                    Box::new(move |new_value| {
                        settings.volume = new_value;
                    })
                )
                .fixed_size(vec2(100.0, 24.0)),
        ]
        .padding_bottom(24.0)
        .padding_right(24.0)
        .align(vec2(1.0, 0.0))
    }
}

pub struct App {
    show_help: bool,
    traffic_counter: traffic::Counter,
    geng: Rc<Geng>,
    resource_pack: ResourcePack,
    assets: Assets,
    framebuffer_size: Vec2<usize>,
    camera: Camera,
    camera_controls: camera::Controls,
    ez3d: Ez3D,
    circle: ugli::VertexBuffer<ez3d::Vertex>,
    connection: Connection,
    player_id: Id,
    view: model::PlayerView,
    tile_mesh: TileMesh,
    noise: noise::OpenSimplex,
    light: light::Uniforms,
    player_positions: HashMap<Id, PlayerData>,
    music: Option<geng::SoundEffect>,
    walk_sound: Option<geng::SoundEffect>,
    ui_state: UiState,
    ui_controller: geng::ui::Controller,
}

impl App {
    pub fn new(
        geng: &Rc<Geng>,
        assets: Assets,
        resource_pack: ResourcePack,
        player_id: Id,
        view: model::PlayerView,
        mut connection: Connection,
    ) -> Self {
        let noise = noise::OpenSimplex::new();
        let light = light::Uniforms::new(&view);
        let tile_mesh = TileMesh::new(geng, &view.tiles, &noise, &resource_pack);
        connection.send(ClientMessage::Ping);
        Self {
            geng: geng.clone(),
            resource_pack,
            assets,
            traffic_counter: traffic::Counter::new(),
            framebuffer_size: vec2(1, 1),
            camera: Camera::new(),
            camera_controls: camera::Controls::new(geng),
            ez3d: Ez3D::new(geng),
            connection,
            player_id,
            view,
            tile_mesh,
            circle: ugli::VertexBuffer::new_static(geng.ugli(), {
                const N: usize = 25;
                (0..=N)
                    .flat_map(|i| {
                        (0..2).map(move |j| ez3d::Vertex {
                            a_pos: Vec2::rotated(
                                vec2(1.0 - j as f32 * 0.1, 0.0),
                                2.0 * f32::PI * i as f32 / N as f32,
                            )
                            .extend(0.0),
                            a_normal: vec3(0.0, 0.0, 0.0),
                            a_emission: 1.0,
                            a_color: Color::WHITE,
                        })
                    })
                    .collect()
            }),
            noise,
            light,
            player_positions: HashMap::new(),
            show_help: true,
            music: None,
            walk_sound: None,
            ui_state: UiState::new(geng),
            ui_controller: geng::ui::Controller::new(),
        }
    }

    fn draw_circle(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        pos: Vec2<f32>,
        radius: f32,
        color: Color<f32>,
    ) {
        let pos = pos.extend(self.tile_mesh.get_height(pos).unwrap_or(0.0));
        self.ez3d.draw_with(
            framebuffer,
            &self.camera,
            &self.light,
            &self.circle,
            std::iter::once(ez3d::Instance {
                i_pos: pos,
                i_size: radius,
                i_rotation: 0.0,
                i_color: color,
            }),
            ugli::DrawMode::TriangleStrip,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
}

impl geng::State for App {
    fn update(&mut self, delta_time: f64) {
        self.ui_controller
            .update(&mut self.ui_state.ui(), delta_time);
        if let Some(music) = &mut self.music {
            music.set_volume(self.ui_state.volume() * 0.3);
        }
        if let Some(sound) = &mut self.walk_sound {
            sound.set_volume(
                self.ui_state.volume() * self.player_positions[&self.player_id].ampl as f64,
            );
        }
        let delta_time = delta_time as f32;

        self.traffic_counter.update(delta_time, &self.connection);

        let mut got_message = false;
        for message in self.connection.new_messages() {
            got_message = true;
            match message {
                ServerMessage::View(view) => {
                    for sound in &view.sounds {
                        let sound = match sound {
                            model::Sound::Craft => &self.assets.craft,
                            model::Sound::PickUp | model::Sound::PutDown => &self.assets.pickup,
                            model::Sound::Hello => {
                                [&self.assets.hello, &self.assets.hi, &self.assets.heyo]
                                    .choose(&mut global_rng())
                                    .unwrap()
                            }
                            model::Sound::StatueGift => &self.assets.blessing,
                        };
                        let mut sound = sound.effect();
                        sound.set_volume(self.ui_state.volume());
                        sound.play();
                    }
                    self.view = view;
                }
                _ => unreachable!(),
            }
        }
        if got_message {
            self.connection.send(ClientMessage::Ping);
        }

        for player in &self.view.players {
            if let Some(prev) = self.player_positions.get_mut(&player.id) {
                prev.update(player, delta_time, &self.view);
            } else {
                self.player_positions
                    .insert(player.id, PlayerData::new(player));
            }
        }
        self.player_positions.retain({
            let view = &self.view;
            move |&id, _| view.players.iter().find(|e| e.id == id).is_some()
        });

        let player_pos = self.player_positions.get(&self.player_id).unwrap().pos;
        self.camera.center += (player_pos
            .extend(self.tile_mesh.get_height(player_pos).unwrap_or(0.0))
            - self.camera.center)
            * (delta_time * 5.0).min(1.0);
        self.camera_controls.update(&mut self.camera, delta_time);
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        self.light = light::Uniforms::new(&self.view);

        ugli::clear(framebuffer, Some(Color::BLACK), Some(1.0));
        self.camera_controls.draw(&mut self.camera, framebuffer);

        self.tile_mesh = TileMesh::new(
            &self.geng,
            &self.view.tiles,
            &self.noise,
            &self.resource_pack,
        );

        self.ez3d.draw(
            framebuffer,
            &self.camera,
            &self.light,
            &self.tile_mesh.mesh,
            std::iter::once(ez3d::Instance {
                i_pos: vec3(0.0, 0.0, 0.0),
                i_rotation: 0.0,
                i_size: 1.0,
                i_color: Color::WHITE,
            }),
        );

        let selected_pos = self
            .tile_mesh
            .intersect(self.camera.pixel_ray(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ))
            .map(|pos| pos.xy());
        let mut selected_item = None;
        let mut selected_player = None;
        if let Some(data) = self.player_positions.get(&self.player_id) {
            self.draw_circle(framebuffer, data.pos, data.size, Color::GREEN);
        }
        if let Some(pos) = selected_pos {
            if let Some((_, item)) = self
                .view
                .items
                .iter()
                .find(|(_, s)| (s.pos - pos).len() <= s.size)
            {
                self.draw_circle(
                    framebuffer,
                    item.pos,
                    item.size,
                    Color::rgba(1.0, 1.0, 1.0, 0.5),
                );
                selected_item = Some(item);
            } else if let Some(player) = self
                .view
                .players
                .iter()
                .find(|e| (e.pos - pos).len() <= e.radius)
            {
                if let Some(data) = self.player_positions.get(&player.id) {
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
        for (_, item) in &self.view.items {
            let pos = item.pos;
            let height = self.tile_mesh.get_height(pos).unwrap_or(0.0);
            let pos = pos.extend(height);
            instances
                .entry(item.item_type.clone())
                .or_default()
                .push(ez3d::Instance {
                    i_pos: pos,
                    i_size: 0.5,
                    i_rotation: self.noise.get([pos.x as f64, pos.y as f64]) as f32 * f32::PI,
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
                .player_positions
                .entry(player.id)
                .or_insert(PlayerData::new(player));
            let mut pos = data.pos.extend(data.step());
            let rotation = data.rotation;
            let height = self
                .tile_mesh
                .get_height(pos.xy())
                .expect("Failed to get player's height");
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
            if let Some(data) = self.player_positions.get(&player.id) {
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

        let player = self
            .view
            .players
            .iter()
            .find(|player| player.id == self.player_id)
            .unwrap();
        let data = &self.player_positions[&player.id];
        if let Some(action) = &player.action {
            match action {
                model::PlayerAction::Crafting { time_left, .. } => {
                    let text = format!("{:.1}", time_left);
                    let pos = data.pos;
                    let pos = pos.extend(self.tile_mesh.get_height(pos).unwrap());
                    self.geng.default_font().draw_aligned(
                        framebuffer,
                        &text,
                        self.camera.world_to_screen(self.framebuffer_size, pos) + vec2(0.0, 50.0),
                        0.5,
                        32.0,
                        Color::WHITE,
                    );
                }
                _ => (),
            }
        }

        self.geng.default_font().draw_aligned(
            framebuffer,
            &format!("SCORE: {}", self.view.score),
            vec2(
                self.framebuffer_size.x as f32 / 2.0,
                self.framebuffer_size.y as f32 - 50.0,
            ),
            0.5,
            48.0,
            Color::WHITE,
        );
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
        if self.show_help {
            let mut y = 32.0;
            let mut text = |text: &str| {
                self.geng
                    .default_font()
                    .draw(framebuffer, text, vec2(32.0, y), 24.0, Color::WHITE);
                y += 24.0;
            };
            let mut found = false;
            for recipe in &self.view.recipes {
                if recipe.is_relevant(self.player_id, &self.view) {
                    found = true;
                    text(&format!("    {}", recipe.to_string()));
                }
            }
            if !found {
                text("    No recipes available");
            }
            text("RECIPES:");
            text("    F to toggle fullscreen");
            text("    Right Mouse Button to interact");
            text("    Left Mouse Button to move");
            text("    H to toggle help");
            text(self.traffic_counter.text());
        }
        self.ui_controller
            .draw(&mut self.ui_state.ui(), framebuffer);
    }
    fn handle_event(&mut self, event: geng::Event) {
        self.ui_controller
            .handle_event(&mut self.ui_state.ui(), event.clone());
        match event {
            geng::Event::MouseDown { position, button } => {
                if self.music.is_none() {
                    self.music = Some({
                        self.assets.music.looped = true;
                        let mut music = self.assets.music.play();
                        music.set_volume(0.2);
                        music
                    });
                    self.walk_sound = Some({
                        self.assets.walk.looped = true;
                        self.assets.walk.play()
                    });
                }
                if let Some(pos) = self.tile_mesh.intersect(
                    self.camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32)),
                ) {
                    let pos = pos.xy();
                    match button {
                        geng::MouseButton::Left => {
                            self.connection.send(ClientMessage::Goto { pos })
                        }
                        geng::MouseButton::Right => {
                            if let Some((id, _)) = self
                                .view
                                .items
                                .iter()
                                .find(|(_, item)| (item.pos - pos).len() <= item.size)
                            {
                                self.connection
                                    .send(ClientMessage::Interact { id: id.clone() })
                            }
                        }
                        _ => {}
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::Q } => {
                let position = self.geng.window().mouse_pos();
                if let Some(pos) = self.tile_mesh.intersect(
                    self.camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32)),
                ) {
                    let pos = pos.xy();
                    self.connection.send(ClientMessage::Drop { pos });
                }
            }
            geng::Event::KeyDown { key: geng::Key::E } => {
                let position = self.geng.window().mouse_pos();
                if let Some(pos) = self.tile_mesh.intersect(
                    self.camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32)),
                ) {
                    let pos = pos.xy();
                    if let Some((id, _)) = self
                        .view
                        .items
                        .iter()
                        .find(|(_, item)| (item.pos - pos).len() <= item.size)
                    {
                        self.connection
                            .send(ClientMessage::PickUp { id: id.clone() });
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.connection.send(ClientMessage::SayHi)
            }
            geng::Event::KeyDown { key: geng::Key::F } => self.geng.window().toggle_fullscreen(),
            geng::Event::KeyDown { key: geng::Key::H } => self.show_help = !self.show_help,
            _ => {}
        }
        self.camera_controls.handle_event(&mut self.camera, &event);
    }
}
