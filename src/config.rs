use crate::model::SortOrder;

#[derive(Clone)]
pub struct Config {
    lazy: bool,
    title_prefix: String,
    sort_order: SortOrder,
    reverse: bool,
    ignore_query_params: bool,
    filter_dir_pattern: Option<String>,
    filter_file_pattern: Option<String>,
}

impl Config {
    pub fn new(
        lazy: bool,
        title_prefix: String,
        sort_order: SortOrder,
        reverse: bool,
        ignore_query_params: bool,
        filter_dir_pattern: Option<String>,
        filter_file_pattern: Option<String>,
    ) -> Self {
        Self {
            lazy,
            title_prefix,
            sort_order,
            reverse,
            ignore_query_params,
            filter_dir_pattern,
            filter_file_pattern,
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

    pub fn reverse(&self) -> bool {
        self.reverse
    }

    pub fn ignore_query_params(&self) -> bool {
        self.ignore_query_params
    }

    pub fn filter_dir_pattern(&self) -> Option<&str> {
        self.filter_dir_pattern.as_deref()
    }

    pub fn filter_file_pattern(&self) -> Option<&str> {
        self.filter_file_pattern.as_deref()
    }
}
