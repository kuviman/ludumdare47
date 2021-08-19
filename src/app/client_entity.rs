use super::*;

#[derive(Deref, DerefMut)]
pub struct ClientEntity {
    #[deref]
    #[deref_mut]
    pub server_entity: model::Entity,
    pub extra_components: ClientEntityComponents,
}

impl ClientEntity {
    pub fn new(server_entity: model::Entity, resource_pack: &ResourcePack) -> Self {
        Self {
            extra_components: {
                let mut components =
                    resource_pack.entity_components[&server_entity.entity_type].clone();
                if let Some(interpolate) = &mut components.interpolate {
                    interpolate.current_pos = server_entity.pos.unwrap();
                }
                components
            },
            server_entity,
        }
    }
    pub fn update_client(&mut self, server_entity: model::Entity) {
        self.server_entity = server_entity;
    }
    pub fn update(&mut self, delta_time: f32, ticks_per_second: f32) {
        if self.extra_components.interpolate.is_some() {
            self.interpolate(delta_time, ticks_per_second);
        }
        if self.extra_components.hopping.is_some() {
            self.hop(delta_time);
        }
    }
    fn interpolate(&mut self, delta_time: f32, ticks_per_second: f32) {
        let entity_pos = self.pos.unwrap();
        let interpolate = self.extra_components.interpolate.as_mut().unwrap();
        let current_pos = interpolate.current_pos;
        interpolate.t += delta_time * 10.0;
        if entity_pos != interpolate.target_pos {
            interpolate.target_pos = entity_pos;
            interpolate.speed = (entity_pos - current_pos).len() / (2.0 / ticks_per_second);
        }
        let dpos = entity_pos - current_pos;
        let delta = dpos.clamp(interpolate.speed * delta_time);
        interpolate.current_pos += delta;
        if dpos.len() > 1e-9 {
            *self.extra_components.rotation.as_mut().unwrap() = dpos.arg();
        }
    }
    fn hop(&mut self, delta_time: f32) {
        let interpolate = self.extra_components.interpolate.as_ref().unwrap();
        let entity_pos = self.pos.unwrap();
        let dpos = entity_pos - interpolate.current_pos;
        let hopping = self.extra_components.hopping.as_mut().unwrap();
        if dpos.len() > 1e-9 {
            hopping.ampl = (hopping.ampl + delta_time * 20.0).min(1.0);
        } else {
            hopping.ampl = (hopping.ampl - delta_time * 20.0).max(0.0);
        }
    }
    pub fn step(&self) -> f32 {
        self.extra_components.hopping.as_ref().unwrap().ampl
            * self
                .extra_components
                .interpolate
                .as_ref()
                .unwrap()
                .t
                .sin()
                .abs()
            * 0.1
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientEntityComponents {
    #[serde(default)]
    pub rotation: Option<f32>,
    #[serde(default)]
    pub renderable: Option<CompRenderable>,
    #[serde(default)]
    pub interpolate: Option<CompInterpolate>,
    #[serde(default)]
    pub hopping: Option<CompHopping>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum CompRenderable {
    Simple,
    Player,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompInterpolate {
    #[serde(default = "CompInterpolate::default_vec2")]
    pub current_pos: Vec2<f32>,
    #[serde(default = "CompInterpolate::default_vec2")]
    target_pos: Vec2<f32>,
    #[serde(default)]
    speed: f32,
    #[serde(default)]
    t: f32,
}

impl CompInterpolate {
    fn default_vec2<T: Default>() -> Vec2<T> {
        vec2(default(), default())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompHopping {
    #[serde(default)]
    pub ampl: f32,
}
