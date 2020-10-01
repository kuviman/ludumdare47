use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Copy, Trans)]
pub struct Id(usize);

impl Id {
    pub fn new() -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        Self(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
    pub fn raw(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Model {
    pub dots: Vec<(Id, Vec2<f32>)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Dot(Vec2<f32>),
}

impl Model {
    pub const TICKS_PER_SECOND: f32 = 1.0;
    pub fn new() -> Self {
        Self { dots: Vec::new() }
    }
    pub fn tick(&mut self) {
        println!("TICK");
    }
    pub fn new_player(&mut self) -> Id {
        Id::new()
    }
    pub fn drop_player(&mut self, player_id: Id) {}
    pub fn handle_message(&mut self, player_id: Id, message: Message) {
        match message {
            Message::Dot(pos) => self.dots.push((player_id, pos)),
        }
    }
}
