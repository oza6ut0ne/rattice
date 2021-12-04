use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    time::Duration,
};

use anyhow::{anyhow, Result};
use askama::Template;
use axum::{
    body::{self, Body, BoxBody, Full},
    extract::ConnectInfo,
    http::{Request, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use tower::ServiceExt;
use tower_http::{auth::RequireAuthorizationLayer, services::ServeDir, trace::TraceLayer};
use tracing::Span;

mod cli;

const IMAGE_EXTTENSIONS: &[&str] = &[
    "apng", "avif", "gif", "jpg", "jpeg", "jfif", "pjpeg", "pjp", "png", "svg", "webp", "bmp",
    "ico", "cur", "tif", "tiff",
];

const VIDEO_EXTTENSIONS: &[&str] = &[
    "3gp", "mpg", "mpeg", "mp4", "m4v", "m4p", "ogv", "ogg", "mov", "webm", "aac", "flac", "mp3",
    "m4a", "oga", "wav",
];

#[tokio::main]
async fn main() -> Result<()> {
    let opt = cli::Opt::init();
    if let Some(path) = &opt.docroot {
        tracing::info!("set document root to {}", path.display());
        std::env::set_current_dir(path)?;
    }

    let mut app = Router::new().nest("/", get(handle_request));

    if opt.username.is_some() || opt.password.is_some() {
        tracing::info!("Basic Authentication enabled");
        app = app.layer(RequireAuthorizationLayer::basic(
            opt.username.unwrap_or_else(|| "".to_owned()).as_str(),
            opt.password.unwrap_or_else(|| "".to_owned()).as_str(),
        ));
    }

    let app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let addr = request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ci| ci.0.to_string())
                    .unwrap_or_else(|| "None".to_owned());
                let encoded_uri = request.uri().path().to_string();
                let decoded_uri =
                    percent_encoding::percent_decode_str(&encoded_uri).decode_utf8_lossy();
                tracing::info_span!("", "{} {} {}", addr, request.method(), decoded_uri)
            })
            .on_request(move |request: &Request<_>, span: &Span| {
                let id: i128 = span.id().map(|i| i.into_u64().into()).unwrap_or(-1);
                tracing::debug!(?id, "started processing request");

                let authorization = request
                    .headers()
                    .get("authorization")
                    .map(|a| a.to_str().unwrap_or(""))
                    .map(|a| a.strip_prefix("Basic ").unwrap_or(""))
                    .map(|a| base64::decode(a).unwrap_or_default())
                    .map(|a| String::from_utf8_lossy(&a).to_string());

                if opt.verbose < 3 {
                    tracing::trace!(?id, "{:?}", request)
                } else if opt.verbose < 4 {
                    tracing::trace!(?authorization, ?id, "{:?}", request)
                } else {
                    tracing::trace!(?authorization, ?id, "{:#?}", request)
                }
            })
            .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
                let id: i128 = span.id().map(|i| i.into_u64().into()).unwrap_or(-1);
                tracing::trace!(?id, "{:?}", response);
                tracing::info!(status = response.status().as_u16(), ?latency, ?id)
            }),
    );

    let addr = format!("{}:{}", opt.bind_address, opt.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("Address is invalid {}:{}", opt.bind_address, opt.port))?;

    if opt.server_cert.is_some() && opt.server_key.is_some() {
        tracing::info!("HTTPS enabled");
        let config =
            RustlsConfig::from_pem_file(opt.server_cert.unwrap(), opt.server_key.unwrap()).await?;
        tracing::info!("listening on {}", addr);
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
            .await?;
    } else {
        tracing::info!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
            .await?;
    }

    Ok(())
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MediaType {
    Image,
    Video,
    Other,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum File {
    Directory {
        path: String,
        name: String,
    },
    File {
        path: String,
        name: String,
        media_type: MediaType,
    },
}

impl MediaType {
    pub fn new(path: &Path) -> Self {
        match path
            .extension()
            .map(|e| e.to_string_lossy().to_string().to_ascii_lowercase())
        {
            Some(ext) => {
                if IMAGE_EXTTENSIONS.contains(&ext.as_str()) {
                    Self::Image
                } else if VIDEO_EXTTENSIONS.contains(&ext.as_str()) {
                    Self::Video
                } else {
                    Self::Other
                }
            }
            None => Self::Other,
        }
    }
}

impl File {
    pub fn new(path_buf: &Path) -> Result<File> {
        let path = path_buf
            .strip_prefix("./")?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert path to &str: {:?}", path_buf))?
            .to_owned();

        let name = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        let file = if path_buf.is_dir() {
            Self::Directory { path, name }
        } else {
            Self::File {
                path,
                name,
                media_type: MediaType::new(path_buf),
            }
        };

        Ok(file)
    }

    pub fn name(&self) -> String {
        match self {
            Self::Directory { path: _, name } => name.clone(),
            Self::File {
                path: _,
                name,
                media_type: _,
            } => name.clone(),
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::File {
                path: _,
                name: _,
                media_type: MediaType::Image,
            }
        )
    }

    pub fn is_video(&self) -> bool {
        matches!(
            self,
            Self::File {
                path: _,
                name: _,
                media_type: MediaType::Video,
            }
        )
    }
}

#[derive(Template)]
#[template(path = "rattice.html")]
struct RatticeTemplate {
    uri: String,
    files: Vec<File>,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response<BoxBody> {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(body::boxed(Full::from(format!(
                    "Failed to render template. Error: {}",
                    err
                ))))
                .unwrap(),
        }
    }
}

enum AppError {
    NotFound,
    InternalServerError(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::InternalServerError(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<BoxBody> {
        match self {
            Self::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::boxed(Full::from("<h1>NOT FOUND</h1>")))
                .unwrap(),
            Self::InternalServerError(e) => {
                tracing::error!("{:?}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(body::boxed(Full::from("<h1>Internal Server Error</h1>")))
                    .unwrap()
            }
        }
    }
}