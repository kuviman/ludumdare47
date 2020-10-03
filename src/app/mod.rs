use super::*;

mod camera;
mod ez;
mod ez3d;
mod tile_mesh;

use camera::Camera;
use ez::Ez;
use ez3d::Ez3D;
use tile_mesh::TileMesh;

#[derive(geng::Assets)]
pub struct Assets {
    tree: ez3d::Obj,
    pebble: ez3d::Obj,
    stick: ez3d::Obj,
}

pub struct App {
    geng: Rc<Geng>,
    assets: Assets,
    framebuffer_size: Vec2<usize>,
    camera: Camera,
    camera_controls: camera::Controls,
    ez: Ez,
    ez3d: Ez3D,
    connection: Connection,
    player_id: Id,
    model: Model,
}

impl App {
    pub fn new(
        geng: &Rc<Geng>,
        assets: Assets,
        player_id: Id,
        model: Model,
        mut connection: Connection,
    ) -> Self {
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
            model,
        }
    }
}

impl geng::State for App {
    fn update(&mut self, delta_time: f64) {
        let mut got_message = false;
        for message in self.connection.new_messages() {
            got_message = true;
            match message {
                ServerMessage::Model(model) => self.model = model,
                _ => unreachable!(),
            }
        }
        self.connection.send(ClientMessage::Ping);
        let delta_time = delta_time as f32;
        self.camera_controls.update(&mut self.camera, delta_time);
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), Some(1.0));
        self.camera_controls.draw(&mut self.camera, framebuffer);

        let view = self.model.get_view(self.player_id);

        let tile_mesh = TileMesh::new(&self.geng, &view.tiles);

        let mut tiles_to_draw = Vec::<(Vec2<usize>, Color<f32>)>::new();

        self.ez3d.draw(
            framebuffer,
            &self.camera,
            &tile_mesh.mesh,
            std::iter::once(ez3d::Instance {
                i_pos: vec3(0.0, 0.0, 0.0),
                i_size: 1.0,
            }),
        );
        for &(obj, structure_type, size) in &[
            (&self.assets.tree, model::StructureType::Tree, 0.5),
            (
                &self.assets.pebble,
                model::StructureType::Item {
                    item: model::Item::Pebble,
                },
                0.2,
            ),
            (
                &self.assets.stick,
                model::StructureType::Item {
                    item: model::Item::Stick,
                },
                0.5,
            ),
        ] {
            self.ez3d.draw(
                framebuffer,
                &self.camera,
                obj.vb(),
                view.structures.iter().filter_map(|e| {
                    let pos = e.pos.map(|x| x as f32 + 0.5);
                    let height = tile_mesh.get_height(pos)?;
                    let pos = pos.extend(height);
                    if e.structure_type == structure_type {
                        Some(ez3d::Instance {
                            i_pos: pos,
                            i_size: size,
                        })
                    } else {
                        None
                    }
                }),
            );
        }
        for structure in view.structures.iter() {
            tiles_to_draw.push((structure.pos, Color::GREEN));
        }
        for entity in view.entities.iter() {
            tiles_to_draw.push((entity.pos, Color::RED));
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
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, button } => {
                let pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if pos.x >= 0.0 && pos.y >= 0.0 {
                    let pos = pos.map(|x| x as usize);
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
            _ => {}
        }
        self.camera_controls.handle_event(&mut self.camera, &event);
    }
}
