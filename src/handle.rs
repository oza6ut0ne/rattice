use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::Query,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::{
    config::Config,
    error::AppError,
    model::File,
    template::{HtmlTemplate, RatticeTemplate},
};

pub fn add_handler(app: Router) -> Router {
    app.nest("/", get(handle_request))
}

async fn handle_request(
    uri: Uri,
    Query(query): Query<HashMap<String, String>>,
    Extension(config): Extension<Arc<Config>>,
) -> Result<Response, AppError> {
    let file_response = serve_file(&uri).await;
    if file_response.is_ok() {
        return file_response;
    }

    let decoded_uri = percent_encoding::percent_decode_str(uri.path()).decode_utf8_lossy();
    let files = list_files(&decoded_uri, &query, &config).map_err(|_| AppError::NotFound)?;
    let template = RatticeTemplate::new(&decoded_uri, files, config.lazy(), config.title_prefix());
    Ok(HtmlTemplate(template).into_response())
}

async fn serve_file(uri: &Uri) -> Result<Response, AppError> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    match ServeDir::new(".")
        .append_index_html_on_directories(false)
        .oneshot(req)
        .await
    {
        Ok(res) => match res.status() {
            StatusCode::NOT_FOUND => Err(AppError::NotFound),
            _ => Ok(res.into_response()),
        },
        Err(e) => Err(anyhow!(e).into()),
    }
}

fn list_files(
    uri: &str,
    query: &HashMap<String, String>,
    config: &Arc<Config>,
) -> Result<Vec<File>> {
    let entries = std::fs::read_dir(format!(".{}", uri))?.collect::<Result<Vec<_>, _>>()?;
    let mut files = entries
        .iter()
        .map(|e| File::new(&e.path(), e.metadata().ok()))
        .collect::<Result<Vec<_>>>()?;

    let order = query.get("order").and_then(|o| o.parse().ok());
    let order = order.as_ref().unwrap_or_else(|| config.sort_order());
    let reverse = query
        .get("reverse")
        .and_then(|r| r.parse().ok())
        .unwrap_or_else(|| config.reverse());

    files.sort_unstable_by(|a, b| a.cmp_by(b, order, reverse));
    if uri != "/" {
        files.insert(
            0,
            File::new_with_name(
                Path::new(&format!(".{}", uri)).parent().unwrap(),
                "..",
                None,
            )?,
        )
    }

    Ok(files)
}
