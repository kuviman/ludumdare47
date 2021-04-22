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
    pub pos: Option<Vec2<f32>>,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub movement_speed: Option<f32>,
    #[serde(default)]
    pub controller: Option<CompController>,
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
    #[serde(default)]
    pub load_area: Option<CompLoadArea>,
    #[serde(default)]
    pub hp: Option<CompHP>,
    #[serde(default)]
    pub weapon: Option<CompWeapon>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct EntityType(pub String);

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Entity {
    pub fn new(
        id: Id,
        entity_type: &EntityType,
        pos: Option<Vec2<f32>>,
        components: &HashMap<EntityType, EntityComponents>,
    ) -> Self {
        let mut components = components[entity_type].clone();
        components.pos = pos;
        if let Some(hp) = &mut components.hp {
            hp.current_hp = hp.max_hp;
        }
        Self {
            entity_type: entity_type.clone(),
            components,
            id,
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
    Attacking {
        target_entity_id: Id,
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
pub struct ActionTarget {
    pub interaction_type: InteractionType,
    pub target_type: TargetType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InteractionType {
    None,
    Interact,
    Attack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TargetType {
    Position { pos: Vec2<f32> },
    Entity { id: Id },
}
