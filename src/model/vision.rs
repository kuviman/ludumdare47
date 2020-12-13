use super::*;

impl Model {
    pub fn add_view_radius(view: &mut HashSet<Vec2<i64>>, center_pos: Vec2<f32>, radius: f32) {
        view.insert(center_pos.map(|x| x as i64).clone());
        for x0 in 1..(radius.round() as i64) {
            view.insert(vec2(x0, 0) + center_pos.map(|x| x as i64));
            view.insert(vec2(center_pos.x as i64 - x0, center_pos.y as i64));
        }
        for y in 1..(radius.round() as i64 + 1) {
            let x = (radius * radius - y as f32 * y as f32).sqrt().round() as i64;
            view.insert(vec2(center_pos.x as i64, center_pos.y as i64 + y));
            view.insert(vec2(center_pos.x as i64, center_pos.y as i64 - y));
            for x0 in 1..x {
                view.insert(vec2(center_pos.x as i64 + x0, center_pos.y as i64 + y));
                view.insert(vec2(center_pos.x as i64 + x0, center_pos.y as i64 - y));
                view.insert(vec2(center_pos.x as i64 - x0, center_pos.y as i64 + y));
                view.insert(vec2(center_pos.x as i64 - x0, center_pos.y as i64 - y));
            }
        }
    }
}
