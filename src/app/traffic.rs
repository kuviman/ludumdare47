use super::*;

pub struct Counter {
    last_tin: usize,
    last_tout: usize,
    next_update: f32,
    text: String,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            last_tin: 0,
            last_tout: 0,
            next_update: 0.0,
            text: String::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, connection: &Connection) {
        self.next_update -= delta_time;
        if self.next_update < 0.0 {
            self.next_update = 1.0;
            self.text = format!(
                "Traffic: {} kb/s in, {} kb/s out",
                (connection.traffic().inbound() - self.last_tin) / 1024,
                (connection.traffic().outbound() - self.last_tout) / 1024,
            );
            self.last_tin = connection.traffic().inbound();
            self.last_tout = connection.traffic().outbound();
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}
