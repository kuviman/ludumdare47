use geng::prelude::*;

mod model;
mod renderer;

use model::{Id, Model};
use renderer::Renderer;

struct App {
    model: Model,
    renderer: Renderer,
    next_tick: f32,
}

impl App {
    pub fn new(geng: &Rc<Geng>) -> Self {
        let mut model = Model::new();
        let renderer = Renderer::new(geng, &mut model);
        Self {
            model,
            renderer,
            next_tick: Model::TICKS_PER_SECOND,
        }
    }
}

impl geng::State for App {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.next_tick -= delta_time;
        while self.next_tick < 0.0 {
            self.next_tick += Model::TICKS_PER_SECOND;
            self.model.tick();
        }
        self.renderer.update(delta_time, &mut self.model);
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.renderer.draw(framebuffer, &mut self.model);
    }
    fn handle_event(&mut self, event: geng::Event) {
        self.renderer.handle_event(event, &mut self.model);
    }
}

fn main() {
    let geng = Rc::new(Geng::new(geng::ContextOptions {
        title: "LudumDare 47".to_owned(),
        ..default()
    }));
    let geng = &geng;
    geng::run(geng.clone(), App::new(geng));
}
