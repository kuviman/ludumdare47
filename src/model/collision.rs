use super::*;

pub enum CollisionResult {
    Undefined,
    NoCollision,
    Collision {
        penetration: f32,
        collision_normal: Vec2<f32>,
    },
}

pub fn entities_collision(
    entity_pos: Vec2<f32>,
    entity_size: f32,
    other_pos: Vec2<f32>,
    other_size: f32,
) -> CollisionResult {
    let dir = entity_pos - other_pos;
    let distance = dir.len();
    if distance == 0.0 {
        CollisionResult::Undefined
    } else if distance <= entity_size + other_size {
        CollisionResult::Collision {
            penetration: entity_size + other_size - distance,
            collision_normal: dir / distance,
        }
    } else {
        CollisionResult::NoCollision
    }
}

pub fn entity_tile_collision(
    entity_pos: Vec2<f32>,
    entity_size: f32,
    tile_pos: Vec2<i64>,
    tile_size: f32,
) -> CollisionResult {
    let up = entity_pos.y - tile_pos.y as f32 - tile_size;
    let down = tile_pos.y as f32 - entity_pos.y;
    let right = entity_pos.x - tile_pos.x as f32 - tile_size;
    let left = tile_pos.x as f32 - entity_pos.x;

    let (dy, ny) = if up.abs() < down.abs() {
        (up, 1.0)
    } else {
        (down, -1.0)
    };
    let (dx, nx) = if right.abs() < left.abs() {
        (right, 1.0)
    } else {
        (left, -1.0)
    };

    // Find direction and distance from the tile to the center point
    let (normal, distance) = if dx <= 0.0 && dy <= 0.0 {
        // Inside
        if dx > dy {
            // Closer to vertical edge
            (vec2(nx, 0.0), dx)
        } else {
            // Closer to horizontal edge
            (vec2(0.0, ny), dy)
        }
    } else if dx <= 0.0 {
        // Outside but closer to horizontal edge
        (vec2(0.0, ny), dy)
    } else if dy <= 0.0 {
        // Outside but closer to vertical edge
        (vec2(nx, 0.0), dx)
    } else {
        // Outside but closer to vertex
        let normal = vec2(nx, ny);
        let normal = normal / normal.len();
        (normal, (dx * dx + dy * dy).sqrt())
    };

    if distance < entity_size {
        CollisionResult::Collision {
            penetration: entity_size - distance,
            collision_normal: normal,
        }
    } else {
        CollisionResult::NoCollision
    }
}
