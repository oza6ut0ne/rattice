use anyhow::anyhow;
use askama::Template;
use axum::{
    body::BoxBody,
    http::Response,
    response::{Html, IntoResponse},
};

use crate::{error::AppError, model::File};

#[derive(Template)]
#[template(path = "rattice.html")]
pub(crate) struct RatticeTemplate {
    pub uri: String,
    pub files: Vec<File>,
}

pub(crate) struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response<BoxBody> {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(e) => AppError::from(anyhow!(e)).into_response(),
        }
    }
}