use std::{collections::HashMap, fs::DirEntry, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Query, RawQuery},
    http::{header::CACHE_CONTROL, Request, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use hyper::HeaderMap;
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::{
    config::Config,
    error::AppError,
    model::{File, FilesContainer, SortOrder},
    template::{HtmlTemplate, RatticeTemplate},
};

pub const REGEX_SIZE_LIMIT: usize = 1024 * 1024;

pub fn add_handler(app: Router) -> Router {
    app.nest_service("/", get(handle_request))
}

async fn handle_request(
    uri: Uri,
    Query(mut query): Query<HashMap<String, String>>,
    RawQuery(mut raw_query): RawQuery,
    headers: HeaderMap,
    Extension(config): Extension<Arc<Config>>,
) -> Result<Response, AppError> {
    let file_response = serve_file(&uri, &headers).await;
    if file_response.is_ok() || !matches!(file_response, Err(AppError::NotFound(_))) {
        return file_response;
    }

    if config.ignore_query_params() {
        query.clear();
        raw_query.take();
    }
    serve_dir(&uri, &query, &raw_query.as_deref(), &config)
}

async fn serve_file(uri: &Uri, headers: &HeaderMap) -> Result<Response, AppError> {
    let mut req = Request::builder().uri(uri);
    let headers_mut = req.headers_mut().unwrap();
    for (k, v) in headers.iter() {
        headers_mut.insert(k, v.to_owned());
    }
    let req = req
        .body(Body::empty())
        .map_err(|e| AppError::BadRequest(e.into()))?;

    match ServeDir::new(".")
        .append_index_html_on_directories(false)
        .oneshot(req)
        .await
    {
        Ok(mut res) => match res.status() {
            StatusCode::NOT_FOUND => Err(AppError::NotFound(anyhow!("ServeDir returned 404"))),
            _ => {
                res.headers_mut()
                    .insert(CACHE_CONTROL, "no-cache".parse().unwrap());
                Ok(res.into_response())
            }
        },
        Err(e) => Err(anyhow!(e).into()),
    }
}

fn serve_dir(
    uri: &Uri,
    query: &HashMap<String, String>,
    raw_query: &Option<&str>,
    config: &Arc<Config>,
) -> Result<Response, AppError> {
    let decoded_uri = percent_encoding::percent_decode_str(uri.path()).decode_utf8_lossy();
    let containers = walk_dir(&decoded_uri, query, config)?;

    let raw_query = raw_query.map(|r| format!("?{}", r)).unwrap_or_default();
    let lazy = query
        .get("lazy")
        .and_then(|r| r.parse().ok())
        .unwrap_or_else(|| config.lazy());

    let template = RatticeTemplate::new(
        &decoded_uri,
        &raw_query,
        containers,
        lazy,
        config.title_prefix(),
        false,
        false,
    );
    Ok(HtmlTemplate(template).into_response())
}

pub(crate) fn walk_dir(
    uri: &str,
    query: &HashMap<String, String>,
    config: &Arc<Config>,
) -> Result<Vec<FilesContainer>, AppError> {
    let re_dir = extract_regex("filter_dir", query, || config.filter_dir_pattern())?;
    let re_file = extract_regex("filter_file", query, || config.filter_file_pattern())?;
    let filter_op = |e: &DirEntry| filter_entry(e, &re_dir, &re_file);

    let order = query.get("order").and_then(|o| o.parse().ok());
    let order = order.as_ref().unwrap_or_else(|| config.sort_order());
    let reverse = query
        .get("reverse")
        .and_then(|r| r.parse().ok())
        .unwrap_or_else(|| config.reverse());

    let depth = query
        .get("depth")
        .and_then(|o| o.parse().ok())
        .unwrap_or(config.depth());

    let mut containers = vec![];
    let mut next_targets = vec![uri.to_owned()];

    for i in 0..depth {
        let mut child_containers = vec![];
        for target_uri in &next_targets {
            let files = list_files(target_uri, order, reverse, filter_op, i == 0)?;
            child_containers.push(FilesContainer::new(target_uri, files))
        }

        if i < depth - 1 {
            next_targets.clear();
            for container in &child_containers {
                let mut urls = container
                    .files()
                    .par_iter()
                    .filter(|f| f.is_dir() && f.name() != "..")
                    .map(|f| f.to_uri())
                    .collect();
                next_targets.append(&mut urls)
            }
        }

        containers.append(&mut child_containers);
        if next_targets.is_empty() {
            break;
        }
    }

    Ok(containers)
}

fn list_files(
    uri: &str,
    order: &SortOrder,
    reverse: bool,
    filter_op: impl Fn(&DirEntry) -> bool + Sync + Send,
    add_parent: bool,
) -> Result<Vec<File>, AppError> {
    let entries = std::fs::read_dir(format!(".{}", uri))
        .map_err(|e| AppError::NotFound(e.into()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::NotFound(e.into()))?;

    let mut files = entries
        .par_iter()
        .filter(|e| filter_op(e))
        .map(|e| File::new(&e.path(), e.metadata().ok()))
        .collect::<Result<Vec<_>>>()?;

    files.par_sort_unstable_by(|a, b| a.cmp_by(b, order, reverse));
    if uri != "/" && add_parent {
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

fn extract_regex<'a>(
    key: &'a str,
    query: &'a HashMap<String, String>,
    default_pattern: impl FnOnce() -> Option<&'a str>,
) -> Result<Option<Regex>, AppError> {
    let pattern = query.get(key).map(|p| p.as_str()).or_else(default_pattern);
    let regex = match pattern {
        Some(pattern) => Some(
            RegexBuilder::new(pattern)
                .size_limit(REGEX_SIZE_LIMIT)
                .build()
                .map_err(|e| AppError::BadRequest(e.into()))?,
        ),
        None => None,
    };
    Ok(regex)
}

fn filter_entry(entry: &DirEntry, regex_dir: &Option<Regex>, regex_file: &Option<Regex>) -> bool {
    if entry.path().is_dir() {
        is_filename_match(entry, regex_dir)
    } else {
        is_filename_match(entry, regex_file)
    }
}

fn is_filename_match(entry: &DirEntry, regex: &Option<Regex>) -> bool {
    if let Some(re) = regex {
        re.is_match(entry.file_name().to_string_lossy().as_ref())
    } else {
        true
    }
}
