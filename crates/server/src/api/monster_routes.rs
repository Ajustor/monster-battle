//! Routes de synchronisation des monstres.

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::monsters::store::{self, ChangeOutcome, MonsterChange, SyncedMonster};

/// Nombre maximum de modifications acceptées dans un seul appel de sync.
const MAX_CHANGES_PER_SYNC: usize = 200;

#[derive(Deserialize)]
pub struct ListQuery {
    /// Ne renvoyer que ce qui a changé après cette date (le `server_time` du
    /// dernier appel).
    since: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct ListResponse {
    monsters: Vec<SyncedMonster>,
    /// Date de référence à repasser au prochain appel.
    server_time: DateTime<Utc>,
}

/// Liste les monstres du joueur, complets ou depuis un point de reprise.
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<ListResponse>> {
    let monsters = store::pull(&state.db, user_id, query.since).await?;
    Ok(Json(ListResponse {
        monsters,
        server_time: Utc::now(),
    }))
}

/// Charge un monstre précis.
pub async fn get_one(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SyncedMonster>> {
    store::get(&state.db, user_id, id)
        .await?
        .filter(|m| !m.deleted)
        .map(Json)
        .ok_or_else(|| ApiError::NotFound("monstre introuvable".to_string()))
}

/// Supprime un monstre (pierre tombale propagée aux autres appareils).
pub async fn delete_one(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = store::soft_delete(&state.db, user_id, id).await?;
    if !deleted {
        return Err(ApiError::NotFound("monstre introuvable".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[derive(Deserialize)]
pub struct SyncRequest {
    /// Point de reprise du client.
    #[serde(default)]
    since: Option<DateTime<Utc>>,
    /// Modifications locales à pousser.
    #[serde(default)]
    changes: Vec<MonsterChange>,
    /// Monstres supprimés localement.
    #[serde(default)]
    deleted: Vec<Uuid>,
}

#[derive(Serialize)]
pub struct SyncResponse {
    /// Sort de chaque modification poussée, dans l'ordre d'envoi.
    results: Vec<ChangeOutcome>,
    /// Ce qui a changé côté serveur, y compris les écritures qu'on vient
    /// d'appliquer — le client peut donc remplacer son cache sans recoller
    /// lui-même les morceaux.
    monsters: Vec<SyncedMonster>,
    server_time: DateTime<Utc>,
}

/// Aller-retour complet : pousse les modifications locales puis renvoie l'état
/// serveur depuis le point de reprise.
pub async fn sync(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<SyncRequest>,
) -> ApiResult<Json<SyncResponse>> {
    if body.changes.len() + body.deleted.len() > MAX_CHANGES_PER_SYNC {
        return Err(ApiError::BadRequest(format!(
            "trop de modifications en un appel (maximum {})",
            MAX_CHANGES_PER_SYNC
        )));
    }

    let mut results = Vec::with_capacity(body.changes.len());
    for change in &body.changes {
        results.push(store::apply(&state.db, user_id, change).await?);
    }

    for id in &body.deleted {
        store::soft_delete(&state.db, user_id, *id).await?;
    }

    let monsters = store::pull(&state.db, user_id, body.since).await?;

    Ok(Json(SyncResponse {
        results,
        monsters,
        server_time: Utc::now(),
    }))
}
