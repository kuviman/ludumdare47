use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct EntityColors {
    pub skin: Color<f32>,
    pub shirt: Color<f32>,
    pub pants: Color<f32>,
}

impl EntityColors {
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
            skin: hsv(0.04, 0.5, global_rng().gen_range(0.25, 1.0)),
            shirt: hsv(global_rng().gen_range(0.0, 1.0), 1.0, 1.0),
            pants: hsv(global_rng().gen_range(0.0, 1.0), 1.0, 1.0),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Entity {
    pub id: Id,
    pub pos: Vec2<usize>,
    pub size: Vec2<usize>,
    pub view_range: f32,
    pub move_to: Option<(Vec2<usize>, bool)>,
    pub item: Option<Item>,
    pub controllable: bool,
    pub colors: EntityColors,
}