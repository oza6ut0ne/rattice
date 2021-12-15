use std::path::Path;

use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
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

async fn handle_request(uri: Uri) -> Result<Response, AppError> {
    let file_response = serve_file(&uri).await;
    if file_response.is_ok() {
        return file_response;
    }
    let encoded_uri = uri.path().to_string();
    let decoded_uri = percent_encoding::percent_decode_str(&encoded_uri).decode_utf8_lossy();
    let files = match list_files(&decoded_uri) {
        Ok(files) => files,
        Err(_) => return Err(AppError::NotFound),
    };
    let template = RatticeTemplate::new(decoded_uri.to_string(), files);
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

fn list_files(uri: &str) -> Result<Vec<File>> {
    let entries = std::fs::read_dir(format!(".{}", uri))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut files = entries
        .iter()
        .map(|e| File::new(e))
        .collect::<Result<Vec<_>>>()?;

    files.sort();
    if uri != "/" {
        files.insert(
            0,
            File::new_with_name(Path::new(&format!(".{}", uri)).parent().unwrap(), "..")?,
        )
    }

    Ok(files)
}
