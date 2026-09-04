//! API HTTP : comptes, synchronisation des monstres.
//!
//! Elle partage le port du relais WebSocket : `main` distingue les deux à
//! partir des premiers octets de la connexion.

pub mod auth_routes;
pub mod monster_routes;

use std::sync::Arc;

use axum::Json;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{delete, get, post};
use axum::{Router, response::IntoResponse};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::Config;

/// État partagé par tous les handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub http: reqwest::Client,
}

/// Construit le routeur de l'API.
pub fn router(state: AppState) -> Router {
    let cors = if state.config.allowed_origins.is_empty() {
        CorsLayer::new().allow_origin(AllowOrigin::any())
    } else {
        let origins: Vec<HeaderValue> = state
            .config
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
    }
    .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/health", get(health))
        .route("/api/v1/auth/providers", get(auth_routes::providers))
        .route("/api/v1/auth/device", get(auth_routes::device_page))
        .route("/api/v1/auth/device/start", post(auth_routes::device_start))
        .route("/api/v1/auth/device/token", post(auth_routes::device_token))
        .route(
            "/api/v1/auth/device/approve",
            post(auth_routes::device_approve),
        )
        .route("/api/v1/auth/refresh", post(auth_routes::refresh))
        .route("/api/v1/auth/logout", post(auth_routes::logout))
        .route(
            "/api/v1/auth/{provider}/start",
            get(auth_routes::oauth_start),
        )
        .route(
            "/api/v1/auth/{provider}/callback",
            get(auth_routes::oauth_callback),
        )
        .route("/api/v1/me", get(auth_routes::me))
        .route("/api/v1/monsters", get(monster_routes::list))
        .route("/api/v1/monsters/{id}", get(monster_routes::get_one))
        .route("/api/v1/monsters/{id}", delete(monster_routes::delete_one))
        .route("/api/v1/sync", post(monster_routes::sync))
        .fallback(not_found)
        .layer(cors)
        .with_state(state)
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    /// `false` quand le serveur tourne en relais seul, sans base de données.
    accounts: bool,
}

async fn health(axum::extract::State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    Json(Health {
        status: "online",
        version: env!("CARGO_PKG_VERSION"),
        accounts: state.config.database_url.is_some(),
    })
}

async fn not_found() -> impl IntoResponse {
    crate::error::ApiError::NotFound("route inconnue".to_string())
}
