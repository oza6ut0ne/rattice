use anyhow::{anyhow, Result};
use axum::{
    body::{Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::error::AppError;
use crate::model::File;
use crate::template::{HtmlTemplate, RatticeTemplate};

pub fn add_handler(app: Router) -> Router {
    app.nest("/", get(handle_request))
}

async fn handle_request(uri: Uri) -> Result<Response<BoxBody>, AppError> {
    let encoded_uri = uri.path().to_string();
    let decoded_uri = percent_encoding::percent_decode_str(&encoded_uri).decode_utf8_lossy();
    let files = match list_files(&decoded_uri) {
        Ok(files) => files,
        Err(_) => return serve_file(uri).await,
    };
    let template = RatticeTemplate {
        uri: decoded_uri.to_string(),
        files,
    };
    Ok(HtmlTemplate(template).into_response())
}

async fn serve_file(uri: Uri) -> Result<Response<BoxBody>, AppError> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    ServeDir::new(".")
        .oneshot(req)
        .await
        .map_err(|e| anyhow!(e).into())
        .map(|res| {
            let res = res.into_response();
            if res.status() == StatusCode::NOT_FOUND {
                AppError::NotFound.into_response()
            } else {
                res
            }
        })
}

fn list_files(uri: &str) -> Result<Vec<File>> {
    let entries = std::fs::read_dir(format!(".{}", uri))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let mut files = entries
        .iter()
        .map(|e| File::new(e))
        .collect::<Result<Vec<_>>>()?;

    files.sort();
    Ok(files)
}