use axum::{
    Router,
    http::{HeaderValue, Method},
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{AppState, routes};

pub fn build_router(state: AppState) -> Router {
    let api = routes::router();
    let static_files =
        ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));

    Router::new()
        .nest("/api", api)
        .fallback_service(static_files)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(HeaderValue::from_static("*"))
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(state)
}
