//! Erreurs de l'API et leur traduction en réponses HTTP.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Erreur renvoyée par un handler de l'API.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("authentification requise")]
    Unauthorized,

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("fonctionnalité indisponible : {0}")]
    Unavailable(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Corps JSON d'une erreur.
#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, details) = match &self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", None),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found", None),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request", None),
            ApiError::Unavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable", None),
            // Les erreurs internes ne fuitent pas leur détail au client, mais
            // sont tracées côté serveur.
            ApiError::Database(e) => {
                eprintln!("❌ Erreur base de données : {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
            ApiError::Internal(e) => {
                eprintln!("❌ Erreur interne : {:#}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
        };

        let message = match status {
            StatusCode::INTERNAL_SERVER_ERROR => "erreur interne du serveur".to_string(),
            _ => self.to_string(),
        };

        (
            status,
            Json(ErrorBody {
                error: code,
                message,
                details,
            }),
        )
            .into_response()
    }
}

/// Raccourci pour les handlers.
pub type ApiResult<T> = Result<T, ApiError>;
