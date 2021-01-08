use super::*;

#[derive(Debug, Deref)]
pub struct Loaded<T> {
    #[deref]
    value: T,
    version: u64,
    loaders: HashMap<Id, Option<u64>>,
}

#[derive(Debug)]
pub enum LoadedUpdate {
    Update,
    Unload,
}

impl<T> Loaded<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            version: 1,
            loaders: HashMap::new(),
        }
    }
    pub fn borrow_mut(&mut self) -> &mut T {
        self.version += 1;
        &mut self.value
    }
    pub fn has_loaders(&self) -> bool {
        !self.loaders.is_empty()
    }
    pub fn unload(&mut self, loader: Id) {
        if let Some(version) = self.loaders.get_mut(&loader) {
            *version = None;
        }
    }
    pub fn forget(&mut self, loader: Id) {
        self.loaders.remove(&loader);
    }
    pub fn load(&mut self, loader: Id) {
        match self.loaders.get(&loader) {
            Some(None) | None => {
                self.loaders.insert(loader, Some(0));
            }
            _ => {}
        }
    }
    pub fn get_update(&mut self, loader: Id) -> Option<LoadedUpdate> {
        match self.loaders.get_mut(&loader) {
            Some(Some(version)) => {
                if *version != self.version {
                    *version = self.version;
                    Some(LoadedUpdate::Update)
                } else {
                    None
                }
            }
            Some(None) => {
                self.loaders.remove(&loader);
                Some(LoadedUpdate::Unload)
            }
            None => None,
        }
    }
}
