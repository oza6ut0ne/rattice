#[derive(Clone)]
pub struct Config {
    lazy: bool,
    title_prefix: String,
}

impl Config {
    pub fn new(lazy: bool, title_prefix: String) -> Self {
        Self {
            lazy,
            title_prefix,
        }
    }

    pub fn lazy(&self) -> bool {
        self.lazy
    }

    pub fn title_prefix(&self) -> &str {
        &self.title_prefix
    }
}
