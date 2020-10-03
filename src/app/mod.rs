use super::*;

mod camera;
mod ez;
mod ez3d;

use camera::Camera;
use ez::Ez;
use ez3d::Ez3D;

pub struct App {
    geng: Rc<Geng>,
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
    pub fn new(geng: &Rc<Geng>, player_id: Id, model: Model, mut connection: Connection) -> Self {
        connection.send(ClientMessage::Ping);
        Self {
            geng: geng.clone(),
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

    fn tile_mesh(tiles: &[model::Tile]) -> Vec<ez3d::Vertex> {
        let mut mesh = Vec::new();
        let tiles: HashMap<Vec2<usize>, model::Tile> =
            tiles.iter().map(|tile| (tile.pos, tile.clone())).collect();
        let mut append = |tile00: &model::Tile| -> Option<()> {
            let tile01 = tiles.get(&(tile00.pos + vec2(0, 1)))?;
            let tile11 = tiles.get(&(tile00.pos + vec2(1, 1)))?;
            let tile10 = tiles.get(&(tile00.pos + vec2(1, 0)))?;
            let a_color = match tile00.ground_type {
                model::GroundType::Water => Color::BLUE,
                model::GroundType::Sand => Color::YELLOW,
            };
            let p00 = tile00.pos.map(|x| x as f32).extend(tile00.height);
            let p01 = tile01.pos.map(|x| x as f32).extend(tile01.height);
            let p11 = tile11.pos.map(|x| x as f32).extend(tile11.height);
            let p10 = tile10.pos.map(|x| x as f32).extend(tile10.height);
            mesh.push(ez3d::Vertex {
                a_pos: p00,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p10,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p11,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p00,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p11,
                a_color,
            });
            mesh.push(ez3d::Vertex {
                a_pos: p01,
                a_color,
            });
            Some(())
        };
        for tile00 in tiles.values() {
            append(tile00);
        }
        mesh
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
        let mut tiles_to_draw = Vec::<(Vec2<usize>, Color<f32>)>::new();

        let view = self.model.get_view(self.player_id);
        for tile in view.tiles.iter() {
            let color = match tile.ground_type {
                model::GroundType::Water => Color::BLUE,
                model::GroundType::Sand => Color::YELLOW,
            };
            tiles_to_draw.push((tile.pos, color));
        }
        self.ez3d.draw(
            framebuffer,
            &self.camera,
            Self::tile_mesh(&view.tiles).into_iter(),
            std::iter::once(ez3d::Instance {
                i_pos: vec3(0.0, 0.0, 0.0),
                i_size: 1.0,
            }),
        );
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
