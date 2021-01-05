use super::*;

#[derive(Deref, DerefMut)]
pub struct Saved<T: Serialize + for<'de> Deserialize<'de>> {
    path: std::path::PathBuf,
    #[deref]
    #[deref_mut]
    value: T,
}

impl<T: Serialize + for<'de> Deserialize<'de>> Saved<T> {
    pub fn new(path: impl AsRef<std::path::Path>, default: impl FnOnce() -> T) -> Self {
        let path = path.as_ref();
        match Self::load(path) {
            Ok(result) => result,
            Err(_) => Self {
                path: path.to_owned(),
                value: default(),
            },
        }
    }
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, anyhow::Error> {
        let path = path.as_ref().to_owned();
        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let value = bincode::deserialize_from(reader)?;
        Ok(Self { path, value })
    }
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let file = std::fs::File::create(&self.path)?;
        let writer = std::io::BufWriter::new(file);
        bincode::serialize_into(writer, &self.value)?;
        Ok(())
    }
}

impl<T: Serialize + for<'de> Deserialize<'de>> Drop for Saved<T> {
    fn drop(&mut self) {
        self.save().unwrap();
    }
}
