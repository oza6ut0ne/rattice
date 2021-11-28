use anyhow::Result;
use askama::Template;
use axum::{
    body::{Body, Bytes, Full},
    extract::ConnectInfo,
    handler::Handler as _,
    http::{Request, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get_service,
    Router,
};
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::Span;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "rattice=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .nest(
            "/static",
            get_service(ServeDir::new(".")).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .fallback(handle_404.into_service())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let addr = request
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .map(|ci| ci.0.to_string())
                        .unwrap_or_else(|| "None".to_owned());
                    tracing::debug_span!("", "{} {} {}", addr, request.method(), request.uri())
                })
                .on_request(|request: &Request<_>, span: &Span| {
                    let id: i128 = span.id().map(|i| i.into_u64().into()).unwrap_or(-1);
                    tracing::debug!(id = ?id, "started processing request");
                    tracing::trace!(id = ?id, "{:?}", request)
                })
                .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
                    let id: i128 = span.id().map(|i| i.into_u64().into()).unwrap_or(-1);
                    tracing::trace!(id = ?id, "{:?}", response);
                    tracing::debug!(
                        id = ?id, latency = ?latency, status = response.status().as_u16(),
                        "finished processing request"
                    )
                }),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
        .await?;

    Ok(())
}

fn list_files(uri: &Uri) -> Result<Vec<String>> {
    let mut entries = std::fs::read_dir(format!(".{}", uri))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort();
    let paths = entries
        .iter()
        .flat_map(|e| e.strip_prefix("./").map(|p| p.to_str().unwrap().to_owned()))
        .collect::<Vec<_>>();
    Ok(paths)
}

async fn handle_404(uri: Uri) -> Result<impl IntoResponse, AppError> {
    let files = match list_files(&uri) {
        Ok(files) => files,
        Err(_) => return Err(AppError::NotFound),
    };
    let template = RatticeTemplate {
        uri: uri.to_string(),
        files,
    };
    Ok(HtmlTemplate(template))
}

#[derive(Template)]
#[template(path = "rattice.html")]
struct RatticeTemplate {
    uri: String,
    files: Vec<String>,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::from(format!(
                    "Failed to render template. Error: {}",
                    err
                )))
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
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            Self::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::from("<h1>NOT FOUND</h1>"))
                .unwrap(),
            Self::InternalServerError(e) => {
                tracing::error!("{:?}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::from("<h1>Internal Server Error</h1>"))
                    .unwrap()
            }
        }
    }
}
