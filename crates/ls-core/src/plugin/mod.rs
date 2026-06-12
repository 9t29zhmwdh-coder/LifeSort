use crate::models::{Classification, FileEntry};

pub trait FilePlugin: Send + Sync {
    fn id(&self) -> &str;
    fn can_handle(&self, entry: &FileEntry) -> bool;
    fn classify(&self, entry: &FileEntry) -> Classification;
    fn display_name(&self) -> &str;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn FilePlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn register(&mut self, plugin: Box<dyn FilePlugin>) {
        self.plugins.push(plugin);
    }

    pub fn find(&self, entry: &FileEntry) -> Option<&dyn FilePlugin> {
        self.plugins.iter().find(|p| p.can_handle(entry)).map(|p| p.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.display_name()).collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
