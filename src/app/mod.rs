use super::*;

mod camera;
mod ez;

use camera::Camera;
use ez::Ez;

pub struct App {
    geng: Rc<Geng>,
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
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.camera_controls.draw(&mut self.camera, framebuffer);
        let mut tiles_to_draw = Vec::new();
        for (y, tiles_row) in self.model.tiles.iter().enumerate() {
            for (x, tile) in tiles_row.iter().enumerate() {
                let color = match tile.ground_type {
                    model::GroundType::Water => Color::BLUE,
                    model::GroundType::Sand => Color::YELLOW,
                };
                tiles_to_draw.push((Vec2::from([x, y]), color));
            }
        }
        for structure in self.model.structures.iter() {
            tiles_to_draw.push((structure.pos, Color::GREEN));
        }
        for entity in self.model.entities.iter() {
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
        self.camera_controls.handle_event(&mut self.camera, &event);
    }
}
