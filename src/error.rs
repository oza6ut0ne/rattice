use axum::{
    body::{self, Full},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::Span;

pub(crate) enum AppError {
    BadRequest(anyhow::Error),
    NotFound(anyhow::Error),
    InternalServerError(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalServerError(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let id: i128 = Span::current()
            .id()
            .map(|i| i.into_u64().into())
            .unwrap_or(-1);
        match self {
            Self::BadRequest(e) => {
                tracing::debug!(?id, "BadRequest: {:?}", e);
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(body::boxed(Full::from("<h1>BAD REQUEST</h1>")))
                    .unwrap()
            }
            Self::NotFound(e) => {
                tracing::debug!(?id, "NotFound: {:?}", e);
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body::boxed(Full::from("<h1>NOT FOUND</h1>")))
                    .unwrap()
            }
            Self::InternalServerError(e) => {
                tracing::error!(?id, "InternalServerError: {:?}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(body::boxed(Full::from("<h1>Internal Server Error</h1>")))
                    .unwrap()
            }
        }
    }
}
