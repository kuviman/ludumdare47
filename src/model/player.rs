use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerColors {
    pub skin: Color<f32>,
    pub shirt: Color<f32>,
    pub pants: Color<f32>,
}

impl PlayerColors {
    pub fn new() -> Self {
        fn hsv(h: f32, s: f32, v: f32) -> Color<f32> {
            let mut h = h;
            h -= h.floor();
            let (r, g, b);
            let f = h * 6.0 - (h * 6.0).floor();
            let p = v * (1.0 - s);
            let q = v * (1.0 - f * s);
            let t = v * (1.0 - (1.0 - f) * s);
            if h * 6.0 < 1.0 {
                r = v;
                g = t;
                b = p;
            } else if h * 6.0 < 2.0 {
                r = q;
                g = v;
                b = p;
            } else if h * 6.0 < 3.0 {
                r = p;
                g = v;
                b = t;
            } else if h * 6.0 < 4.0 {
                r = p;
                g = q;
                b = v;
            } else if h * 6.0 < 5.0 {
                r = t;
                g = p;
                b = v;
            } else {
                r = v;
                g = p;
                b = q;
            }
            Color::rgb(r, g, b)
        }
        Self {
            skin: hsv(0.04, 0.5, global_rng().gen_range(0.25..=1.0)),
            shirt: hsv(global_rng().gen_range(0.0..1.0), 1.0, 1.0),
            pants: hsv(global_rng().gen_range(0.0..1.0), 1.0, 1.0),
        }
    }
}

impl Default for PlayerColors {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerAction {
    MovingTo {
        target: MovementTarget,
        finish_action: Option<MomentAction>,
    },
    Crafting {
        item_id: Id,
        recipe: Recipe,
        time_left: f32,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MovementTarget {
    Position { pos: Vec2<f32> },
    Entity { id: Id },
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum MomentAction {
    Interact { id: Id },
    Drop { pos: Vec2<f32> },
    PickUp { id: Id },
}
