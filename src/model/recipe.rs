use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub ingredient1: Option<EntityType>,
    pub ingredient2: Option<EntityType>,
    pub result1: Option<EntityType>,
    pub result2: Option<EntityType>,
    pub conditions: Option<Biome>,
    pub craft_time: f32,
}

impl Recipe {
    pub fn ingredients_equal(
        &self,
        ingredient1: Option<EntityType>,
        ingredient2: Option<EntityType>,
        conditions: Option<Biome>,
    ) -> bool {
        ingredient1 == self.ingredient1
            && ingredient2 == self.ingredient2
            && match self.conditions {
                None => true,
                _ => conditions == self.conditions,
            }
    }
    pub fn is_relevant(&self, player_id: Id, view: &ClientView) -> bool {
        self.ingredient1.as_ref()
            == view
                .entities
                .iter()
                .find(|p| p.id == player_id)
                .and_then(|p| p.components.player.as_ref().unwrap().item.as_ref())
    }
    pub fn to_string(&self) -> String {
        format!(
            "{} + {} = {}{}",
            self.ingredient1
                .as_ref()
                .map_or("Empty Hand".to_owned(), |item| item.to_string()),
            self.ingredient2
                .as_ref()
                .map_or("Empty Space".to_owned(), |s| s.to_string()),
            if self.result1 == self.ingredient1 || self.result1.is_none() {
                self.result2
                    .as_ref()
                    .map_or("None".to_owned(), |s| s.to_string())
            } else {
                if self.ingredient2 == self.result2 || self.result2.is_none() {
                    self.result1
                        .as_ref()
                        .map_or("None".to_owned(), |s| s.to_string())
                } else {
                    format!(
                        "{} + {}",
                        self.result1.as_ref().unwrap().to_string(),
                        self.result2.as_ref().unwrap().to_string()
                    )
                }
            },
            if let Some(biome) = &self.conditions {
                format!(" (only in {})", biome)
            } else {
                "".to_owned()
            }
        )
    }
}
