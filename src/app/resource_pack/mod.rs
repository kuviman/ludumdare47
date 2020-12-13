use super::*;

#[derive(Serialize, Deserialize)]
pub struct BiomeRendering {
    pub color: Color<f32>,
}

pub struct ResourcePack {
    pub biomes: HashMap<model::Biome, BiomeRendering>,
}

impl ResourcePack {
    pub fn empty() -> Self {
        Self {
            biomes: HashMap::new(),
        }
    }
    pub fn merge(&mut self, other: ResourcePack) {
        self.biomes.extend(other.biomes);
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
