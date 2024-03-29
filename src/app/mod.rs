use super::*;

mod camera;
mod client_entity;
mod draw;
mod ez3d;
mod light;
mod resource_pack;
mod tile_mesh;
mod traffic;

use camera::Camera;
use client_entity::ClientEntity;
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
    geng: Geng,
    settings: AutoSave<Settings>,
    volume_slider: geng::ui::Slider,
}

impl UiState {
    fn new(geng: &Geng) -> Self {
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
    traffic_counter: traffic::Counter,
    geng: Geng,
    resource_pack: Rc<ResourcePack>,
    assets: Assets,
    framebuffer_size: Vec2<usize>,
    camera: Camera,
    camera_controls: camera::Controls,
    ez3d: Rc<Ez3D>,
    circle: ugli::VertexBuffer<ez3d::Vertex>,
    connection: Connection,
    player_id: Id,
    view: model::ClientView,
    entities: HashMap<Id, ClientEntity>,
    tile_mesh: TileMesh,
    light: light::Uniforms,
    music: Option<geng::SoundEffect>,
    walk_sound: Option<geng::SoundEffect>,
    ui_state: UiState,
    ui_controller: geng::ui::Controller,
}

impl App {
    pub fn new(
        geng: &Geng,
        assets: Assets,
        resource_pack: &Rc<ResourcePack>,
        player_id: Id,
        view: model::ClientView,
        mut connection: Connection,
    ) -> Self {
        let ez3d = Rc::new(Ez3D::new(geng));
        let ez3d = &ez3d;
        let light = light::Uniforms::new(&view);
        let tile_mesh = TileMesh::new(geng, ez3d, resource_pack);
        connection.send(ClientMessage::RequestUpdate { load_area: None });
        Self {
            geng: geng.clone(),
            resource_pack: resource_pack.clone(),
            assets,
            traffic_counter: traffic::Counter::new(),
            framebuffer_size: vec2(1, 1),
            camera: Camera::new(),
            camera_controls: camera::Controls::new(geng),
            ez3d: ez3d.clone(),
            connection,
            player_id,
            view,
            entities: HashMap::new(),
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
            light,
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
            let mut play = false;
            if let Some(entity) = self.entities.get(&self.player_id) {
                if let Some(hopping) = &entity.extra_components.hopping {
                    sound.set_volume(self.ui_state.volume() * hopping.ampl as f64);
                    play = true;
                }
            }
            if !play {
                sound.set_volume(0.0);
            }
        }
        let delta_time = delta_time as f32;

        self.traffic_counter.update(delta_time, &self.connection);

        let mut request_update = false;
        for message in self.connection.new_messages() {
            match message {
                ServerMessage::UpdateClientView(view) => {
                    request_update = true;
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
                ServerMessage::UpdateTiles(tiles) => {
                    self.tile_mesh.update(&tiles);
                }
                ServerMessage::UnloadArea(area) => {
                    self.tile_mesh.unload(area);
                }
                _ => unreachable!(),
            }
        }

        for server_entity in &self.view.entities {
            if let Some(entity) = self.entities.get_mut(&server_entity.id) {
                entity.update_client(server_entity.clone());
                entity.update(delta_time, self.view.ticks_per_second);
            } else {
                self.entities.insert(
                    server_entity.id,
                    ClientEntity::new(server_entity.clone(), &self.resource_pack),
                );
            }
        }

        let player_pos = self.entities[&self.player_id].pos.unwrap();

        if request_update {
            let load_radius = self.camera.distance;
            self.connection.send(ClientMessage::RequestUpdate {
                load_area: Some(AABB::from_corners(
                    player_pos - vec2(load_radius, load_radius),
                    player_pos + vec2(load_radius, load_radius),
                )),
            });
        }

        self.entities.retain({
            let view = &self.view;
            move |&id, _| view.entities.iter().find(|e| e.id == id).is_some()
        });

        self.camera.center += (player_pos
            .extend(self.tile_mesh.get_height(player_pos).unwrap_or(0.0))
            - self.camera.center)
            * (delta_time * 5.0).min(1.0);
        self.camera_controls.update(&mut self.camera, delta_time);
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw(framebuffer);
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
                            if let Some(entity) = self.view.get_closest_entity(pos) {
                                self.connection.send(ClientMessage::Interact {
                                    target: model::ActionTarget {
                                        interaction_type: model::InteractionType::Interact,
                                        target_type: model::TargetType::Entity { id: entity.id },
                                    },
                                })
                            }
                        }
                        _ => {}
                    }
                }
            }
            geng::Event::KeyDown {
                key: geng::Key::Space,
            } => {
                let position = self.geng.window().mouse_pos();
                if let Some(pos) = self.tile_mesh.intersect(
                    self.camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32)),
                ) {
                    let pos = pos.xy();
                    if let Some(entity) = self.view.get_closest_entity(pos) {
                        self.connection.send(ClientMessage::Interact {
                            target: model::ActionTarget {
                                interaction_type: model::InteractionType::Attack,
                                target_type: model::TargetType::Entity { id: entity.id },
                            },
                        });
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
                    if let Some(entity) = self.view.get_closest_entity(pos) {
                        self.connection
                            .send(ClientMessage::PickUp { id: entity.id });
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.connection.send(ClientMessage::SayHi)
            }
            geng::Event::KeyDown { key: geng::Key::F } => self.geng.window().toggle_fullscreen(),
            _ => {}
        }
        self.camera_controls.handle_event(&mut self.camera, &event);
    }
}
