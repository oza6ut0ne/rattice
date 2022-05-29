use crate::model::SortOrder;

#[derive(Clone)]
pub struct Config {
    lazy: bool,
    title_prefix: String,
    sort_order: SortOrder,
}

impl Config {
    pub fn new(lazy: bool, title_prefix: String, sort_order: SortOrder) -> Self {
        Self {
            lazy,
            title_prefix,
            sort_order,
        }
    }

    pub fn lazy(&self) -> bool {
        self.lazy
    }

    pub fn title_prefix(&self) -> &str {
        &self.title_prefix
    }

    pub fn sort_order(&self) -> &SortOrder {
        &self.sort_order
    }
}
