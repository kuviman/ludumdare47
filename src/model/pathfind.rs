use super::*;

impl Model {
    pub fn pathfind(&self, from: Vec2<usize>, to: Vec2<usize>) -> Option<Vec2<usize>> {
        let mut queue = std::collections::VecDeque::new();
        let mut used = HashSet::new();
        queue.push_back(to);
        used.insert(to);
        let mut max_iterations = 1000;
        while let Some(pos) = queue.pop_front() {
            max_iterations -= 1;
            if max_iterations == 0 {
                break;
            }
            for dx in -1..=1isize {
                for dy in -1..=1isize {
                    let next = pos.map(|x| x as isize) + vec2(dx, dy);
                    if 0 <= next.x
                        && 0 <= next.y
                        && next.x < self.size.x as isize
                        && next.y < self.size.y as isize
                    {
                        let next = next.map(|x| x as usize);
                        if next == from {
                            return Some(pos);
                        }
                        if !used.contains(&next) {
                            let tile = self.get_tile(next).unwrap();
                            if Biome::Water != tile.biome && self.is_traversable_tile(next) {
                                used.insert(next);
                                queue.push_back(next);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
