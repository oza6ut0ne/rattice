use axum::Router;
use tower_http::auth::RequireAuthorizationLayer;

pub fn add_basic_authentication(
    app: Router,
    username: &Option<String>,
    password: &Option<String>,
) -> Router {
    app.layer(RequireAuthorizationLayer::basic(
        username.as_deref().unwrap_or(""),
        password.as_deref().unwrap_or(""),
    ))
}
