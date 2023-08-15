use axum::Router;
use tower_http::validate_request::ValidateRequestHeaderLayer;

pub fn add_basic_authentication(
    app: Router,
    username: &Option<String>,
    password: &Option<String>,
) -> Router {
    app.layer(ValidateRequestHeaderLayer::basic(
        username.as_deref().unwrap_or(""),
        password.as_deref().unwrap_or(""),
    ))
}
