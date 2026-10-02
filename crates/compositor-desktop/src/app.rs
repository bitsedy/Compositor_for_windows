pub struct CompositorApp {
    pub name: String,
}

impl CompositorApp {
    pub fn new() -> Self {
        Self {
            name: "Compositor".to_string(),
        }
    }
}

impl Default for CompositorApp {
    fn default() -> Self {
        Self::new()
    }
}
