use super::*;
use client_entity::ClientEntityComponents;

#[derive(Serialize, Deserialize)]
pub struct BiomeRendering {
    pub color: Color<f32>,
}

pub struct EntityRendering {
    pub model: ez3d::Obj,
}

#[derive(Serialize, Deserialize)]
struct EntityInfo {
    model: String,
}

pub struct ResourcePack {
    pub biomes: HashMap<model::Biome, BiomeRendering>,
    pub entities: HashMap<model::EntityType, EntityRendering>,
    pub entity_components: HashMap<model::EntityType, ClientEntityComponents>,
}

impl ResourcePack {
    pub fn empty() -> Self {
        Self {
            biomes: HashMap::new(),
            entities: HashMap::new(),
            entity_components: HashMap::new(),
        }
    }
    pub fn merge(&mut self, other: ResourcePack) {
        self.biomes.extend(other.biomes);
        self.entities.extend(other.entities);
        self.entity_components.extend(other.entity_components);
    }
    async fn load(geng: &Geng, name: &str) -> Result<Self, anyhow::Error> {
        let path = format!("packs/{}/client", name);
        Ok(Self {
            biomes: {
                let data =
                    <String as geng::LoadAsset>::load(geng, &format!("{}/biomes.json", path))
                        .await?;
                serde_json::from_str(&data)?
            },
            entities: {
                let mut models = HashMap::new();
                if let Ok(data) =
                    <String as geng::LoadAsset>::load(geng, &format!("{}/models.json", path)).await
                {
                    let entities_info: HashMap<model::EntityType, EntityInfo> =
                        serde_json::from_str(&data)?;
                    for (entity_type, entity_info) in entities_info {
                        let model = <ez3d::Obj as geng::LoadAsset>::load(
                            geng,
                            &format!("{}/entities/{}", path, entity_info.model),
                        )
                        .await?;
                        models.insert(entity_type.clone(), EntityRendering { model });
                    }
                }
                models
            },
            entity_components: {
                let data =
                    <String as geng::LoadAsset>::load(geng, &format!("{}/entities.json", path))
                        .await?;
                serde_json::from_str(&data)?
            },
        })
    }
    pub async fn load_all(geng: Geng, pack_list: Vec<String>) -> Result<Self, anyhow::Error> {
        let mut result = Self::empty();
        for pack in pack_list {
            result.merge(Self::load(&geng, &pack).await?);
        }
        Ok(result)
    }
}
