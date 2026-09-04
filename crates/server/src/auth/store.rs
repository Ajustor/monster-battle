//! Accès base pour les comptes, identités OAuth, sessions et appairages.
//!
//! Les requêtes utilisent la forme fonction de `sqlx` (et non les macros
//! `query!`) : la compilation ne dépend donc pas d'une base accessible, ce qui
//! garde la CI simple.

use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::oauth::ProviderIdentity;
use super::tokens;

/// Un joueur.
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Trouve le compte lié à une identité OAuth, ou le crée.
///
/// Si l'identité est inconnue mais que son email vérifié correspond à celui
/// d'une identité existante, les deux sont rattachées au même compte : se
/// connecter avec GitHub puis Google retombe sur le même joueur.
pub async fn upsert_user_from_identity(
    pool: &PgPool,
    identity: &ProviderIdentity,
) -> sqlx::Result<User> {
    let mut tx = pool.begin().await?;

    let existing =
        sqlx::query("SELECT user_id FROM identities WHERE provider = $1 AND provider_user_id = $2")
            .bind(identity.provider.as_str())
            .bind(&identity.provider_user_id)
            .fetch_optional(&mut *tx)
            .await?;

    let user_id = match existing {
        Some(row) => {
            let user_id: Uuid = row.get("user_id");
            // Le nom et l'avatar peuvent avoir changé chez le fournisseur.
            sqlx::query(
                "UPDATE users
                 SET display_name = $2, avatar_url = COALESCE($3, avatar_url), last_seen_at = now()
                 WHERE id = $1",
            )
            .bind(user_id)
            .bind(&identity.display_name)
            .bind(&identity.avatar_url)
            .execute(&mut *tx)
            .await?;
            user_id
        }
        None => {
            // Rattachement par email vérifié, sinon création d'un compte.
            let linked = match &identity.email {
                Some(email) => sqlx::query(
                    "SELECT user_id FROM identities
                     WHERE email IS NOT NULL AND lower(email) = lower($1)
                     LIMIT 1",
                )
                .bind(email)
                .fetch_optional(&mut *tx)
                .await?
                .map(|row| row.get::<Uuid, _>("user_id")),
                None => None,
            };

            let user_id = match linked {
                Some(id) => id,
                None => {
                    let id = Uuid::new_v4();
                    sqlx::query(
                        "INSERT INTO users (id, display_name, avatar_url) VALUES ($1, $2, $3)",
                    )
                    .bind(id)
                    .bind(&identity.display_name)
                    .bind(&identity.avatar_url)
                    .execute(&mut *tx)
                    .await?;
                    id
                }
            };

            sqlx::query(
                "INSERT INTO identities (id, user_id, provider, provider_user_id, email)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(identity.provider.as_str())
            .bind(&identity.provider_user_id)
            .bind(&identity.email)
            .execute(&mut *tx)
            .await?;

            user_id
        }
    };

    let row =
        sqlx::query("SELECT id, display_name, avatar_url, created_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;

    tx.commit().await?;

    Ok(User {
        id: row.get("id"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        created_at: row.get("created_at"),
    })
}

/// Charge un joueur par son identifiant.
pub async fn find_user(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<User>> {
    let row =
        sqlx::query("SELECT id, display_name, avatar_url, created_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(row.map(|row| User {
        id: row.get("id"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        created_at: row.get("created_at"),
    }))
}

// ── Sessions ──────────────────────────────────────────────────────────

/// Crée un refresh token et retourne son secret en clair (seule occasion de
/// le lire : la base n'en garde que l'empreinte).
pub async fn issue_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    device_label: Option<&str>,
    ttl: std::time::Duration,
) -> sqlx::Result<String> {
    let secret = tokens::random_secret();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, device_label, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(tokens::hash_secret(&secret))
    .bind(device_label)
    .bind(Utc::now() + Duration::seconds(ttl.as_secs() as i64))
    .execute(pool)
    .await?;
    Ok(secret)
}

/// Consomme un refresh token et en émet un nouveau (rotation).
///
/// La rotation fait qu'un jeton volé cesse de fonctionner dès que le client
/// légitime rafraîchit sa session.
pub async fn rotate_refresh_token(
    pool: &PgPool,
    secret: &str,
    ttl: std::time::Duration,
) -> sqlx::Result<Option<(Uuid, String)>> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        "UPDATE refresh_tokens
         SET revoked_at = now()
         WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()
         RETURNING user_id, device_label",
    )
    .bind(tokens::hash_secret(secret))
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(None);
    };

    let user_id: Uuid = row.get("user_id");
    let device_label: Option<String> = row.get("device_label");

    let new_secret = tokens::random_secret();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, device_label, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(tokens::hash_secret(&new_secret))
    .bind(&device_label)
    .bind(Utc::now() + Duration::seconds(ttl.as_secs() as i64))
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE users SET last_seen_at = now() WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Some((user_id, new_secret)))
}

/// Révoque un refresh token (déconnexion d'un appareil).
pub async fn revoke_refresh_token(pool: &PgPool, secret: &str) -> sqlx::Result<bool> {
    let result = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now()
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(tokens::hash_secret(secret))
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

// ── Appairage d'appareil (RFC 8628) ───────────────────────────────────

/// Demande d'appairage fraîchement créée.
#[derive(Debug, Clone)]
pub struct DeviceAuthRequest {
    /// Secret que le client garde pour lui et présente en boucle.
    pub device_code: String,
    /// Code court que le joueur recopie dans le navigateur.
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
}

/// Ouvre une demande d'appairage pour un client sans navigateur.
pub async fn create_device_request(
    pool: &PgPool,
    device_label: Option<&str>,
    ttl: std::time::Duration,
) -> sqlx::Result<DeviceAuthRequest> {
    let id = Uuid::new_v4();
    let device_code = tokens::random_secret();
    let display_code = tokens::random_user_code();
    let expires_at = Utc::now() + Duration::seconds(ttl.as_secs() as i64);

    sqlx::query(
        "INSERT INTO device_auth_requests
             (id, device_code_hash, user_code, device_label, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(tokens::hash_secret(&device_code))
    .bind(tokens::normalize_user_code(&display_code))
    .bind(device_label)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(DeviceAuthRequest {
        device_code,
        user_code: display_code,
        expires_at,
    })
}

/// Retrouve une demande d'appairage encore valide à partir du code saisi.
pub async fn find_pending_device_request(
    pool: &PgPool,
    user_code: &str,
) -> sqlx::Result<Option<Uuid>> {
    let row = sqlx::query(
        "SELECT id FROM device_auth_requests
         WHERE user_code = $1 AND approved_at IS NULL AND expires_at > now()",
    )
    .bind(tokens::normalize_user_code(user_code))
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| row.get("id")))
}

/// Marque une demande d'appairage comme approuvée par un joueur.
pub async fn approve_device_request(
    pool: &PgPool,
    request_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<bool> {
    let result = sqlx::query(
        "UPDATE device_auth_requests
         SET user_id = $2, approved_at = now()
         WHERE id = $1 AND approved_at IS NULL AND expires_at > now()",
    )
    .bind(request_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// État d'une demande d'appairage vue par le client qui interroge le serveur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevicePollOutcome {
    /// Le joueur n'a pas encore approuvé : réessayer plus tard.
    Pending,
    /// Approuvé : le client peut ouvrir sa session.
    Approved(Uuid),
    /// Code inconnu, expiré, ou déjà échangé contre des jetons.
    Expired,
}

/// Consomme une demande approuvée. Un `device_code` ne donne des jetons qu'une
/// seule fois : rejouer le même code retourne `Expired`.
pub async fn poll_device_request(
    pool: &PgPool,
    device_code: &str,
) -> sqlx::Result<DevicePollOutcome> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        "SELECT id, user_id, approved_at, consumed_at, expires_at
         FROM device_auth_requests
         WHERE device_code_hash = $1
         FOR UPDATE",
    )
    .bind(tokens::hash_secret(device_code))
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(DevicePollOutcome::Expired);
    };

    let id: Uuid = row.get("id");
    let user_id: Option<Uuid> = row.get("user_id");
    let approved_at: Option<DateTime<Utc>> = row.get("approved_at");
    let consumed_at: Option<DateTime<Utc>> = row.get("consumed_at");
    let expires_at: DateTime<Utc> = row.get("expires_at");

    let outcome = match (approved_at, user_id, consumed_at) {
        _ if expires_at <= Utc::now() => DevicePollOutcome::Expired,
        (_, _, Some(_)) => DevicePollOutcome::Expired,
        (Some(_), Some(user_id), None) => {
            sqlx::query("UPDATE device_auth_requests SET consumed_at = now() WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            DevicePollOutcome::Approved(user_id)
        }
        _ => DevicePollOutcome::Pending,
    };

    tx.commit().await?;
    Ok(outcome)
}

// ── États OAuth ───────────────────────────────────────────────────────

/// Enregistre un état anti-CSRF, éventuellement lié à un appairage en cours.
pub async fn create_oauth_state(
    pool: &PgPool,
    provider: &str,
    device_auth_id: Option<Uuid>,
    redirect_to: Option<&str>,
    ttl: std::time::Duration,
) -> sqlx::Result<String> {
    let state = tokens::random_secret();
    sqlx::query(
        "INSERT INTO oauth_states (state, provider, device_auth_id, redirect_to, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&state)
    .bind(provider)
    .bind(device_auth_id)
    .bind(redirect_to)
    .bind(Utc::now() + Duration::seconds(ttl.as_secs() as i64))
    .execute(pool)
    .await?;
    Ok(state)
}

/// État OAuth consommé au retour du fournisseur.
#[derive(Debug, Clone)]
pub struct OAuthState {
    pub provider: String,
    pub device_auth_id: Option<Uuid>,
    pub redirect_to: Option<String>,
}

/// Consomme un état OAuth : il n'est utilisable qu'une fois.
pub async fn consume_oauth_state(pool: &PgPool, state: &str) -> sqlx::Result<Option<OAuthState>> {
    let row = sqlx::query(
        "DELETE FROM oauth_states
         WHERE state = $1 AND expires_at > now()
         RETURNING provider, device_auth_id, redirect_to",
    )
    .bind(state)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| OAuthState {
        provider: row.get("provider"),
        device_auth_id: row.get("device_auth_id"),
        redirect_to: row.get("redirect_to"),
    }))
}

/// Purge les demandes d'appairage et états OAuth expirés.
pub async fn purge_expired(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM oauth_states WHERE expires_at < now()")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM device_auth_requests WHERE expires_at < now() - interval '1 day'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < now() - interval '30 days'")
        .execute(pool)
        .await?;
    Ok(())
}
