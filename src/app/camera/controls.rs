use super::*;

pub struct Controls {
    geng: Rc<Geng>,
    framebuffer_size: Vec2<usize>,
    previous_mouse: Vec2<f32>,
    moving: bool,
    rotating: Option<Vec2<f32>>,
}

impl Controls {
    pub const ATTACK_RANGE: RangeInclusive<f32> = 0.2..=f32::PI / 2.0;
    pub const DISTANCE_RANGE: RangeInclusive<f32> = 4.0..=1024.0;
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            framebuffer_size: vec2(1, 1),
            previous_mouse: vec2(0.0, 0.0),
            moving: false,
            rotating: None,
        }
    }
    pub fn draw(&mut self, camera: &mut Camera, framebuffer: &mut ugli::Framebuffer) {
        #![allow(unused_variables)]
        self.framebuffer_size = framebuffer.size();
    }
    pub fn update(&mut self, camera: &mut Camera, delta_time: f32) {
        #![allow(unused_variables)]
    }
    pub fn handle_event(&mut self, camera: &mut Camera, event: &geng::Event) {
        match *event {
            geng::Event::MouseDown { position, button } => {
                self.previous_mouse = position.map(|x| x as f32);
                match button {
                    geng::MouseButton::Right
                        if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
                    {
                        self.rotating = Some(self.previous_mouse);
                    }
                    geng::MouseButton::Right => self.moving = true,
                    geng::MouseButton::Middle => self.rotating = Some(self.previous_mouse),
                    _ => {}
                }
            }
            geng::Event::MouseMove { position } => {
                let position = position.map(|x| x as f32);
                if self.moving {
                    let p1 = camera.screen_to_world(self.framebuffer_size, self.previous_mouse);
                    let p2 = camera.screen_to_world(self.framebuffer_size, position);
                    camera.center += p1 - p2;
                }
                if self.rotating.is_some() {
                    camera.rotation +=
                        2.0 * (position.x - self.previous_mouse.x) / self.framebuffer_size.y as f32;
                    camera.attack = clamp(
                        camera.attack
                            - 2.0 * (position.y - self.previous_mouse.y)
                                / self.framebuffer_size.y as f32,
                        Self::ATTACK_RANGE,
                    );
                }
                self.previous_mouse = position;
            }
            geng::Event::MouseUp { button, .. } => match button {
                geng::MouseButton::Right | geng::MouseButton::Middle => {
                    self.moving = false;
                    self.rotating = None;
                }
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                let delta = delta as f32;
                camera.distance = clamp(
                    camera.distance / (delta / 300.0).exp2(),
                    Self::DISTANCE_RANGE,
                );
            }
            geng::Event::KeyDown { key } => match key {
                geng::Key::O => camera.perspective = !camera.perspective,
                geng::Key::I => *camera = Camera::new(),
                _ => {}
            },
            _ => {}
        }
    }
}
