use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Deref, DerefMut)]
pub struct Entity {
    pub entity_type: EntityType,
    pub id: Id,
    #[deref]
    #[deref_mut]
    pub components: EntityComponents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityComponents {
    #[serde(default)]
    pub renderable: Option<CompRenderable>,
    #[serde(default)]
    pub pos: Option<Vec2<f32>>,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub controller: Option<CompController>,
    #[serde(default)]
    pub player: Option<CompPlayer>,
    #[serde(default)]
    pub collidable: Option<CompCollidable>,
    #[serde(default)]
    pub pickable: Option<CompPickable>,
    #[serde(default)]
    pub holding: Option<CompHolding>,
    #[serde(default)]
    pub interaction: Option<CompInteraction>,
    #[serde(default)]
    pub action: Option<CompAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct EntityType(pub String);

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Entity {
    pub fn new(entity_type: &EntityType, components: EntityComponents, id: Id) -> Self {
        Self {
            entity_type: entity_type.clone(),
            id,
            components,
        }
    }

    pub fn move_towards(&mut self, target_pos: Vec2<f32>, movement_speed: f32, delta_time: f32) {
        let entity_pos = self.pos.unwrap();
        let dir = target_pos - entity_pos;
        let dir = dir / dir.len();
        let new_pos = entity_pos + dir * movement_speed * delta_time;
        self.pos = Some(new_pos);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityAction {
    MovingTo {
        target: ActionTarget,
    },
    Crafting {
        target: ActionTarget,
        recipe: Recipe,
        time_left: f32,
    },
    Interact {
        target: ActionTarget,
    },
    Drop {
        pos: Vec2<f32>,
    },
    PickUp {
        id: Id,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ActionTarget {
    Position { pos: Vec2<f32>, target_size: f32 },
    Entity { id: Id },
}
