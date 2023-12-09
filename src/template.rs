use anyhow::anyhow;
use askama::Template;
use axum::response::{Html, IntoResponse, Response};

use crate::{
    error::AppError,
    model::{File, FilesContainer},
};

#[derive(Template)]
#[template(path = "rattice.html")]
pub(crate) struct RatticeTemplate<'a> {
    uri: &'a str,
    query: &'a str,
    containers: Vec<FilesContainer>,
    lazy: bool,
    title_prefix: &'a str,
}

impl<'a> RatticeTemplate<'a> {
    pub fn new(
        uri: &'a str,
        query: &'a str,
        containers: Vec<FilesContainer>,
        lazy: bool,
        title_prefix: &'a str,
    ) -> Self {
        Self {
            uri,
            query,
            containers,
            lazy,
            title_prefix,
        }
    }
}

pub(crate) struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::from(anyhow!(e)).into_response(),
        }
    }
}
