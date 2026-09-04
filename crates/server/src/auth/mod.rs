//! Comptes joueurs : jetons, fournisseurs OAuth, appairage d'appareil.

pub mod jwt;
pub mod oauth;
pub mod store;
pub mod tokens;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::api::AppState;
use crate::error::ApiError;

/// Extracteur exigeant un jeton d'accès valide.
///
/// À utiliser en argument de handler : sa seule présence protège la route.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser(pub Uuid);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        let claims =
            jwt::decode(&state.config.jwt_secret, token.trim()).ok_or(ApiError::Unauthorized)?;

        Ok(AuthUser(claims.sub))
    }
}
