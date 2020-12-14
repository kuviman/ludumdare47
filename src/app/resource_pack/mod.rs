use super::*;

#[derive(Serialize, Deserialize)]
pub struct BiomeRendering {
    pub color: Color<f32>,
}

pub struct ItemRendering {
    pub model: ez3d::Obj,
}

#[derive(Serialize, Deserialize)]
struct ItemInfo {
    model: String,
}

pub struct ResourcePack {
    pub biomes: HashMap<model::Biome, BiomeRendering>,
    pub items: HashMap<model::ItemType, ItemRendering>,
}

impl ResourcePack {
    pub fn empty() -> Self {
        Self {
            biomes: HashMap::new(),
            items: HashMap::new(),
        }
    }
    pub fn merge(&mut self, other: ResourcePack) {
        self.biomes.extend(other.biomes);
        self.items.extend(other.items);
    }
    async fn load(geng: &Rc<Geng>, name: &str) -> Result<Self, anyhow::Error> {
        let path = format!("packs/{}/client", name);
        Ok(Self {
            biomes: {
                let data =
                    <String as geng::LoadAsset>::load(geng, &format!("{}/biomes.json", path))
                        .await?;
                serde_json::from_str(&data)?
            },
            items: {
                let mut items = HashMap::new();
                if let Ok(data) =
                    <String as geng::LoadAsset>::load(geng, &format!("{}/items.json", path)).await
                {
                    let items_info: HashMap<model::ItemType, ItemInfo> =
                        serde_json::from_str(&data)?;
                    for (item_type, item_info) in items_info {
                        let model = <ez3d::Obj as geng::LoadAsset>::load(
                            geng,
                            &format!("{}/items/{}", path, item_info.model),
                        )
                        .await?;
                        items.insert(item_type, ItemRendering { model });
                    }
                }
                items
            },
        })
    }
    pub async fn load_all(geng: Rc<Geng>, pack_list: Vec<String>) -> Result<Self, anyhow::Error> {
        let mut result = Self::empty();
        for pack in pack_list {
            result.merge(Self::load(&geng, &pack).await?);
        }
        Ok(result)
    }
}
