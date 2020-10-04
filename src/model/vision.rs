use super::*;

impl Model {
    pub fn calc_view_range(&self) -> f32 {
        let time = self.current_time as f32;
        let day = self.day_length as f32;
        let night = self.night_length as f32;
        let mut t = 2.0 * (time - day - night / 2.0).abs() / (day + night);
        if t > 1.0 {
            t = 2.0 - t;
        }
        self.rules.entity_night_view_distance
            + t * (self.rules.entity_day_view_distance - self.rules.entity_night_view_distance)
                as f32
    }
    pub fn add_view_radius(view: &mut HashSet<Vec2<usize>>, center_pos: Vec2<usize>, radius: f32) {
        view.insert(center_pos.clone());
        for x0 in 1..(radius.round() as usize) {
            view.insert(vec2(x0, 0) + center_pos);
            view.insert(vec2(center_pos.x - x0, center_pos.y));
        }
        for y in 1..(radius.round() as usize + 1) {
            let x = (radius * radius - y as f32 * y as f32).sqrt().round() as usize;
            view.insert(vec2(center_pos.x, center_pos.y + y));
            view.insert(vec2(center_pos.x, center_pos.y - y));
            for x0 in 1..x {
                view.insert(vec2(center_pos.x + x0, center_pos.y + y));
                view.insert(vec2(center_pos.x + x0, center_pos.y - y));
                view.insert(vec2(center_pos.x - x0, center_pos.y + y));
                view.insert(vec2(center_pos.x - x0, center_pos.y - y));
            }
        }
    }
    pub fn is_under_view(&self, pos: Vec2<usize>) -> bool {
        self.entities.values().any(|entity| {
            let dx = pos.x as f32 - entity.pos.x as f32;
            let dy = pos.y as f32 - entity.pos.y as f32;
            let dist_sqr = dx * dx + dy * dy;
            dist_sqr <= entity.view_range * entity.view_range + 0.5
        }) || self.structures.iter().any(|structure| {
            let dx = pos.x as f32 - structure.pos.x as f32;
            let dy = pos.y as f32 - structure.pos.y as f32;
            let dist_sqr = dx * dx + dy * dy;
            structure.structure_type == StructureType::Campfire
                && dist_sqr <= self.rules.campfire_light * self.rules.campfire_light + 0.5
                || structure.structure_type == StructureType::Item { item: Item::Torch }
                    && dist_sqr <= self.rules.torch_light * self.rules.torch_light + 0.5
        })
    }
}
