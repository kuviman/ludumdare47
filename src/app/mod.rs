use super::*;

use noise::NoiseFn;

mod camera;
mod ez;
mod ez3d;
mod light;
mod tile_mesh;

use camera::Camera;
use ez::Ez;
use ez3d::Ez3D;
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
    tree: ez3d::Obj,
    pebble: ez3d::Obj,
    stick: ez3d::Obj,
    player: PlayerAssets,
    axe: ez3d::Obj,
    campfire: ez3d::Obj,
    black_cloud: ez3d::Obj,
    double_stick: ez3d::Obj,
    log: ez3d::Obj,
    planks: ez3d::Obj,
    raft: ez3d::Obj,
    torch: ez3d::Obj,
    gold_nugget: ez3d::Obj,
    gold_pickaxe: ez3d::Obj,
    gold_rock: ez3d::Obj,
    big_mushroom: ez3d::Obj,
    pickaxe: ez3d::Obj,
    rock: ez3d::Obj,
    sharp_stone: ez3d::Obj,
    shovel: ez3d::Obj,
    magic_crystal: ez3d::Obj,
    crystal_shard: ez3d::Obj,
    statue: ez3d::Obj,
}

impl Assets {
    fn item(&self, item: model::Item) -> &ez3d::Obj {
        match item {
            model::Item::Axe => &self.axe,
            model::Item::DoubleStick => &self.double_stick,
            model::Item::Log => &self.log,
            model::Item::Pebble => &self.pebble,
            model::Item::Planks => &self.planks,
            model::Item::Stick => &self.stick,
            model::Item::Torch => &self.torch,
            model::Item::GoldNugget => &self.gold_nugget,
            model::Item::GoldPickaxe => &self.gold_pickaxe,
            model::Item::SharpStone => &self.sharp_stone,
            model::Item::Shovel => &self.shovel,
            model::Item::CrystalShard => &self.crystal_shard,
            model::Item::Pickaxe => &self.pickaxe,
            _ => &self.black_cloud,
        }
    }
    fn structure(&self, structure: model::StructureType) -> &ez3d::Obj {
        match structure {
            model::StructureType::Tree => &self.tree,
            model::StructureType::Campfire => &self.campfire,
            model::StructureType::Raft => &self.raft,
            model::StructureType::Rock => &self.rock,
            model::StructureType::GoldRock => &self.gold_rock,
            model::StructureType::BigMushroom => &self.big_mushroom,
            model::StructureType::MagicCrystal => &self.magic_crystal,
            model::StructureType::Item { item } => self.item(item),
            _ => &self.black_cloud,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Rafted {
    Not,
    Jumping,
    On,
}

struct EntityData {
    pos: Vec2<f32>,
    target_pos: Vec2<usize>,
    speed: f32,
    rotation: f32,
    ampl: f32,
    t: f32,
    rafted: Rafted,
}

impl EntityData {
    fn new(entity: &model::Entity) -> Self {
        Self {
            pos: entity.pos.map(|x| x as f32 + 0.5),
            speed: 0.0,
            rotation: 0.0,
            target_pos: entity.pos,
            ampl: 0.0,
            t: 0.0,
            rafted: Rafted::Not,
        }
    }
    fn step(&self) -> f32 {
        self.ampl * self.t.sin().abs() * 0.1
    }
    fn update(&mut self, entity: &model::Entity, rafted: bool, tick_time: f32) {
        self.t += tick_time * 5.0;
        if entity.pos != self.target_pos {
            self.target_pos = entity.pos;
            self.speed = (entity.pos.map(|x| x as f32 + 0.5) - self.pos).len();
            if rafted {
                if self.rafted == Rafted::Not {
                    self.rafted = Rafted::Jumping;
                } else {
                    self.rafted = Rafted::On;
                }
            } else {
                self.rafted = Rafted::Not;
            }
        }
        let dpos = entity.pos.map(|x| x as f32 + 0.5) - self.pos;
        self.pos += dpos.clamp(self.speed * tick_time);
        if dpos.len() > 0.5 {
            self.rotation = dpos.arg();
        }
        if dpos.len() > 0.01 {
            self.ampl = (self.ampl + tick_time * 10.0).min(1.0);
        } else {
            self.ampl = (self.ampl - tick_time * 10.0).max(0.0);
        }
    }
}

struct BlackCloud {
    size: f32,
    rotation: f32,
    rotation_speed: f32,
}

impl Default for BlackCloud {
    fn default() -> Self {
        Self {
            size: 0.0,
            rotation: global_rng().gen_range(0.0, 2.0 * f32::PI),
            rotation_speed: global_rng().gen_range(-1.0, 1.0),
        }
    }
}

pub struct App {
    geng: Rc<Geng>,
    assets: Assets,
    framebuffer_size: Vec2<usize>,
    camera: Camera,
    camera_controls: camera::Controls,
    ez: Ez,
    ez3d: Ez3D,
    pentagon: ugli::VertexBuffer<ez3d::Vertex>,
    connection: Connection,
    player_id: Id,
    view: model::PlayerView,
    tile_mesh: TileMesh,
    noise: noise::OpenSimplex,
    light: light::Uniforms,
    entity_positions: HashMap<Id, EntityData>,
    black_clouds: HashMap<Vec2<usize>, BlackCloud>,
}

impl App {
    pub fn new(
        geng: &Rc<Geng>,
        assets: Assets,
        player_id: Id,
        view: model::PlayerView,
        mut connection: Connection,
    ) -> Self {
        let noise = noise::OpenSimplex::new();
        let light = light::Uniforms::new(&view);
        let tile_mesh = TileMesh::new(geng, &view.tiles, &view.height_map, &noise);
        connection.send(ClientMessage::Ping);
        Self {
            geng: geng.clone(),
            assets,
            framebuffer_size: vec2(1, 1),
            camera: Camera::new(),
            camera_controls: camera::Controls::new(geng),
            ez: Ez::new(geng),
            ez3d: Ez3D::new(geng),
            connection,
            player_id,
            view,
            tile_mesh,
            pentagon: ugli::VertexBuffer::new_static(geng.ugli(), {
                const N: usize = 5;
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
            entity_positions: HashMap::new(),
            black_clouds: HashMap::new(),
        }
    }

    fn draw_pentagon(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        pos: Vec2<f32>,
        color: Color<f32>,
    ) {
        let pos = pos.extend(self.tile_mesh.get_height(pos).unwrap());
        self.ez3d.draw_with(
            framebuffer,
            &self.camera,
            &self.light,
            &self.pentagon,
            std::iter::once(ez3d::Instance {
                i_pos: pos,
                i_size: 0.5,
                i_rotation: self.noise.get([pos.x as f64, pos.y as f64]) as f32,
                i_color: color,
            }),
            ugli::DrawMode::TriangleStrip,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }

    fn update_black_clouds(&mut self, delta_time: f32) {
        let mut positions = HashSet::new();
        for tile in &self.view.tiles {
            for dx in 0..3 {
                for dy in 0..3 {
                    positions.insert(tile.pos + vec2(dx, dy));
                }
            }
        }
        for tile in &self.view.tiles {
            positions.remove(&(tile.pos + vec2(1, 1)));
        }
        for &pos in &positions {
            self.black_clouds.entry(pos).or_default();
        }
        for (&pos, cloud) in &mut self.black_clouds {
            if positions.contains(&pos) {
                cloud.size = 0.8;
            } else {
                cloud.size = 0.0;
            }
            cloud.rotation += cloud.rotation_speed * delta_time;
        }
        self.black_clouds.retain(|_, cloud| cloud.size > 0.0);
    }
}

impl geng::State for App {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        let mut got_message = false;
        for message in self.connection.new_messages() {
            got_message = true;
            match message {
                ServerMessage::View(view) => self.view = view,
                _ => unreachable!(),
            }
        }
        if got_message {
            self.connection.send(ClientMessage::Ping);
        }

        for entity in &self.view.entities {
            if let Some(prev) = self.entity_positions.get_mut(&entity.id) {
                prev.update(
                    entity,
                    self.view
                        .tiles
                        .iter()
                        .find(|tile| tile.pos == entity.pos)
                        .unwrap()
                        .biome
                        == model::Biome::Water,
                    delta_time * self.view.ticks_per_second,
                );
            } else {
                self.entity_positions
                    .insert(entity.id, EntityData::new(entity));
            }
        }
        self.entity_positions.retain({
            let view = &self.view;
            move |&id, _| view.entities.iter().find(|e| e.id == id).is_some()
        });

        self.update_black_clouds(delta_time);

        self.camera.center += (self.entity_positions.get(&self.player_id).unwrap().pos
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
            &self.view.height_map,
            &self.noise,
        );

        let mut tiles_to_draw = Vec::<(Vec2<usize>, Color<f32>)>::new();

        self.ez3d.draw(
            framebuffer,
            &self.camera,
            &self.light,
            self.assets.black_cloud.vb(),
            self.black_clouds
                .iter()
                .map(|(&pos, cloud)| ez3d::Instance {
                    i_pos: pos.map(|x| x as f32 - 0.5).extend(
                        self.view.height_map[pos.y.min(self.view.height_map.len() - 1)]
                            [pos.x.min(self.view.height_map[0].len() - 1)],
                    ),
                    i_rotation: cloud.rotation,
                    i_size: cloud.size,
                    i_color: Color::BLACK,
                }),
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

        if let Some(data) = self.entity_positions.get(&self.player_id) {
            self.draw_pentagon(framebuffer, data.pos, Color::GREEN);
        }
        let selected_pos = self
            .tile_mesh
            .intersect(self.camera.pixel_ray(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ))
            .map(|pos| pos.xy().map(|x| x as usize));
        if let Some(pos) = selected_pos {
            self.draw_pentagon(
                framebuffer,
                pos.map(|x| x as f32 + 0.5),
                Color::rgba(1.0, 1.0, 1.0, 0.5),
            );
        }

        let mut instances: HashMap<model::StructureType, Vec<ez3d::Instance>> = HashMap::new();
        for structure in &self.view.structures {
            let pos = structure.pos.map(|x| x as f32 + 0.5);
            let height = self.tile_mesh.get_height(pos).unwrap();
            let pos = pos.extend(height);
            instances
                .entry(structure.structure_type)
                .or_default()
                .push(ez3d::Instance {
                    i_pos: pos,
                    i_size: 0.5,
                    i_rotation: self.noise.get([pos.x as f64, pos.y as f64]) as f32 * f32::PI,
                    i_color: Color::WHITE,
                });
        }
        for (structure_type, instances) in instances {
            let obj = self.assets.structure(structure_type);
            self.ez3d.draw(
                framebuffer,
                &self.camera,
                &self.light,
                obj.vb(),
                instances.into_iter(),
            );
        }
        for entity in &self.view.entities {
            let data = self
                .entity_positions
                .entry(entity.id)
                .or_insert(EntityData::new(entity));
            let mut pos = data.pos.extend(data.step());
            let rotation = data.rotation;
            let height = self
                .tile_mesh
                .get_height(pos.xy())
                .expect("Failed to get player's height");
            pos.z += height;
            pos.z = pos.z.max(0.0);
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
                    i_color: entity.colors.skin,
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
                    i_color: entity.colors.shirt,
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
                    i_color: entity.colors.pants,
                }),
            );
            let raft_pos = match data.rafted {
                Rafted::Not => None,
                Rafted::Jumping => Some(data.target_pos.map(|x| x as f32 + 0.5).extend(0.0)),
                Rafted::On => Some(pos),
            };
            if let Some(raft_pos) = raft_pos {
                self.ez3d.draw(
                    framebuffer,
                    &self.camera,
                    &self.light,
                    self.assets.raft.vb(),
                    std::iter::once(ez3d::Instance {
                        i_pos: raft_pos,
                        i_size: 0.5,
                        i_rotation: -rotation,
                        i_color: Color::WHITE,
                    }),
                );
            }
            if let Some(item) = entity.item {
                self.ez3d.draw(
                    framebuffer,
                    &self.camera,
                    &self.light,
                    self.assets.item(item).vb(),
                    std::iter::once(ez3d::Instance {
                        i_pos: pos + Vec2::rotated(vec2(0.45, 0.0), rotation).extend(0.6),
                        i_size: 0.5,
                        i_rotation: -rotation,
                        i_color: Color::WHITE,
                    }),
                );
            }
        }
        self.ez.quads(
            framebuffer,
            &self.camera,
            tiles_to_draw.into_iter().map(|(pos, color)| ez::Quad {
                pos: pos.map(|x| x as f32 + 0.5),
                rotation: 0.0,
                size: vec2(0.5, 0.5) * 0.5,
                color: Color {
                    r: color.r / 2.0,
                    g: color.g / 2.0,
                    b: color.b / 2.0,
                    a: 0.5,
                },
            }),
        );
        if let Some(pos) = selected_pos {
            if let Some(struc) = self.view.structures.iter().find(|s| s.pos == pos) {
                let text = match struc.structure_type {
                    model::StructureType::Item { item } => format!("{:?}", item),
                    _ => format!("{:?}", struc.structure_type),
                };
                let pos = pos.map(|x| x as f32 + 0.5);
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
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, button } => {
                if let Some(pos) = self.tile_mesh.intersect(
                    self.camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32)),
                ) {
                    let pos = pos.xy().map(|x| x as usize);
                    match button {
                        geng::MouseButton::Left => self.connection.send(ClientMessage::Click {
                            pos,
                            secondary: false,
                        }),
                        geng::MouseButton::Right => self.connection.send(ClientMessage::Click {
                            pos,
                            secondary: true,
                        }),
                        _ => {}
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::F } => self.geng.window().toggle_fullscreen(),
            _ => {}
        }
        self.camera_controls.handle_event(&mut self.camera, &event);
    }
}
