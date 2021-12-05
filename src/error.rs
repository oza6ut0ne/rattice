use axum::{
    body::{self, BoxBody, Full},
    http::{Response, StatusCode},
    response::IntoResponse,
};

pub(crate) enum AppError {
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
