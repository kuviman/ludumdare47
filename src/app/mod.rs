use super::*;

mod camera;
mod ez;

use camera::Camera;
use ez::Ez;

pub struct App {
    geng: Rc<Geng>,
    framebuffer_size: Vec2<usize>,
    camera: Camera,
    camera_controls: camera::Controls,
    ez: Ez,
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
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.camera_controls.draw(&mut self.camera, framebuffer);
        let mut tiles_to_draw = Vec::new();

        let vision = self.model.get_view(self.player_id);
        for tile in vision.tiles.iter() {
            let color = match tile.ground_type {
                model::GroundType::Water => Color::BLUE,
                model::GroundType::Sand => Color::YELLOW,
            };
            tiles_to_draw.push((tile.pos, color));
        }
        for structure in vision.structures.iter() {
            tiles_to_draw.push((structure.pos, Color::GREEN));
        }
        for entity in vision.entities.iter() {
            tiles_to_draw.push((entity.pos, Color::RED));
        }
        self.ez.quads(
            framebuffer,
            &self.camera,
            tiles_to_draw.into_iter().map(|(pos, color)| ez::Quad {
                pos: pos.map(|x| x as f32 + 0.5),
                rotation: 0.0,
                size: vec2(0.5, 0.5),
                color,
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
