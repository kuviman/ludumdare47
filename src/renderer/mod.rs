use super::*;

pub struct Renderer {
    geng: Rc<Geng>,
    player_id: Id,
}

impl Renderer {
    pub fn new(geng: &Rc<Geng>, model: &mut Model) -> Self {
        Self {
            geng: geng.clone(),
            player_id: model.new_player(),
        }
    }
    pub fn update(&mut self, delta_time: f32, model: &mut Model) {}
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, model: &mut Model) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        for &(_, pos) in &model.dots {
            self.geng
                .draw_2d()
                .circle(framebuffer, pos, 10.0, Color::RED);
        }
    }
    pub fn handle_event(&mut self, event: geng::Event, model: &mut Model) {
        match event {
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                position,
            } => {
                let position = position.map(|x| x as f32);
                model.handle_message(self.player_id, model::Message::Dot(position))
            }
            _ => {}
        }
    }
}
