use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Trans)]
pub struct Recipe {
    pub ingredient1: Option<Item>,
    pub ingredient2: Option<StructureType>,
    pub result1: Option<Item>,
    pub result2: Option<StructureType>,
    pub conditions: Option<Biome>,
}

impl Recipe {
    pub fn ingredients_equal(
        &self,
        ingredient1: Option<Item>,
        ingredient2: Option<StructureType>,
        conditions: Biome,
    ) -> bool {
        ingredient1 == self.ingredient1
            && ingredient2 == self.ingredient2
            && match &self.conditions {
                None => true,
                Some(cond) => *cond == conditions,
            }
    }
}
