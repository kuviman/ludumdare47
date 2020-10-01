use super::*;

mod camera;
mod ez;

use camera::Camera;
use ez::Ez;

pub struct Renderer {
    geng: Rc<Geng>,
    player_id: Id,
    camera: Camera,
    camera_controls: camera::Controls,
    ez: Ez,
}

impl Renderer {
    fn draw_tile(&self, framebuffer: &mut ugli::Framebuffer, tile: Vec2<usize>, color: Color<f32>) {
        self.ez.quads(
            framebuffer,
            &self.camera,
            std::iter::once(ez::Quad {
                pos: tile.map(|x| x as f32 + 0.5),
                rotation: 0.0,
                size: vec2(0.5, 0.5),
                color,
            }),
        );
    }
}

impl Renderer {
    pub fn new(geng: &Rc<Geng>, model: &mut Model) -> Self {
        Self {
            geng: geng.clone(),
            player_id: model.new_player(),
            camera: Camera::new(),
            camera_controls: camera::Controls::new(geng),
            ez: Ez::new(geng),
        }
    }
    pub fn update(&mut self, delta_time: f32, model: &mut Model) {
        self.camera_controls.update(&mut self.camera, delta_time);
    }
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, model: &mut Model) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.camera_controls.draw(&mut self.camera, framebuffer);
        for (y, tiles_row) in model.tiles.iter().enumerate() {
            for (x, tile) in tiles_row.iter().enumerate() {
                let color = match tile {
                    model::Tile::Water => Color::BLUE,
                    model::Tile::Sand => Color::YELLOW,
                };
                self.draw_tile(framebuffer, Vec2::from([x, y]), color);
            }
        }
    }
    pub fn handle_event(&mut self, event: geng::Event, model: &mut Model) {
        self.camera_controls.handle_event(&mut self.camera, &event);
    }
}
