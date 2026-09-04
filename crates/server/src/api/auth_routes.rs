//! Routes d'authentification.
//!
//! Deux parcours mènent à une session :
//!
//! * **Navigateur** (site de l'arène) : `/auth/{provider}/start` → consentement
//!   chez le fournisseur → `/auth/{provider}/callback` → jetons.
//! * **Client sans navigateur** (TUI, mobile) : `POST /auth/device/start` donne
//!   un code court ; le joueur l'approuve depuis un navigateur ; le client
//!   récupère ses jetons via `POST /auth/device/token`.

use std::str::FromStr;
use std::time::Duration;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::auth::oauth::{self, Provider};
use crate::auth::store::{self, DevicePollOutcome};
use crate::auth::{AuthUser, jwt};
use crate::config::OAuthCredentials;
use crate::error::{ApiError, ApiResult};

/// Durée de vie d'une demande d'appairage : assez pour ouvrir un navigateur,
/// assez court pour qu'un code affiché à l'écran ne traîne pas.
const DEVICE_CODE_TTL: Duration = Duration::from_secs(10 * 60);

/// Intervalle de sondage conseillé au client.
const DEVICE_POLL_INTERVAL_SECS: u64 = 3;

/// Durée de vie d'un état anti-CSRF OAuth.
const OAUTH_STATE_TTL: Duration = Duration::from_secs(10 * 60);

/// Jetons remis à un client authentifié.
#[derive(Serialize)]
pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user_id: uuid::Uuid,
    pub display_name: String,
}

/// Récupère les identifiants OAuth d'un fournisseur, ou explique qu'il n'est
/// pas configuré sur ce serveur.
fn credentials(state: &AppState, provider: Provider) -> ApiResult<&OAuthCredentials> {
    let credentials = match provider {
        Provider::GitHub => state.config.github.as_ref(),
        Provider::Google => state.config.google.as_ref(),
    };
    credentials.ok_or_else(|| {
        ApiError::Unavailable(format!(
            "la connexion via {} n'est pas configurée sur ce serveur",
            provider
        ))
    })
}

fn parse_provider(name: &str) -> ApiResult<Provider> {
    Provider::from_str(name)
        .map_err(|_| ApiError::NotFound(format!("fournisseur inconnu : {}", name)))
}

/// Émet une paire de jetons pour un joueur.
async fn open_session(
    state: &AppState,
    user_id: uuid::Uuid,
    device_label: Option<&str>,
) -> ApiResult<SessionTokens> {
    let user = store::find_user(&state.db, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("compte introuvable".to_string()))?;

    let access_token = jwt::encode(
        &state.config.jwt_secret,
        user_id,
        state.config.access_token_ttl,
    )
    .map_err(ApiError::Internal)?;

    let refresh_token = store::issue_refresh_token(
        &state.db,
        user_id,
        device_label,
        state.config.refresh_token_ttl,
    )
    .await?;

    Ok(SessionTokens {
        access_token,
        refresh_token,
        expires_in: state.config.access_token_ttl.as_secs(),
        user_id,
        display_name: user.display_name,
    })
}

// ── Découverte ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ProvidersResponse {
    providers: Vec<&'static str>,
}

/// Liste les fournisseurs réellement configurés, pour que les clients
/// n'affichent pas un bouton qui échouerait.
pub async fn providers(State(state): State<AppState>) -> Json<ProvidersResponse> {
    let mut providers = Vec::new();
    if state.config.github.is_some() {
        providers.push(Provider::GitHub.as_str());
    }
    if state.config.google.is_some() {
        providers.push(Provider::Google.as_str());
    }
    Json(ProvidersResponse { providers })
}

// ── Parcours navigateur ───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct OAuthStartQuery {
    /// Code d'appairage en cours, quand la connexion sert à valider un client
    /// sans navigateur.
    user_code: Option<String>,
    /// URL de retour après connexion (site de l'arène).
    redirect_to: Option<String>,
}

/// Démarre un aller-retour OAuth.
pub async fn oauth_start(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthStartQuery>,
) -> ApiResult<Redirect> {
    let provider = parse_provider(&provider)?;
    let credentials = credentials(&state, provider)?;

    // Un code d'appairage invalide est signalé tout de suite : inutile
    // d'envoyer le joueur chez GitHub pour échouer au retour.
    let device_auth_id = match &query.user_code {
        Some(code) => Some(
            store::find_pending_device_request(&state.db, code)
                .await?
                .ok_or_else(|| {
                    ApiError::BadRequest("code d'appairage inconnu ou expiré".to_string())
                })?,
        ),
        None => None,
    };

    let redirect_to = query
        .redirect_to
        .as_deref()
        .filter(|url| is_allowed_redirect(&state, url));

    let csrf_state = store::create_oauth_state(
        &state.db,
        provider.as_str(),
        device_auth_id,
        redirect_to,
        OAUTH_STATE_TTL,
    )
    .await?;

    Ok(Redirect::to(&provider.authorization_url(
        credentials,
        &state.config.redirect_uri(provider.as_str()),
        &csrf_state,
    )))
}

/// N'accepte comme URL de retour qu'une origine explicitement autorisée : sans
/// cela le serveur servirait de tremplin à redirection ouverte.
fn is_allowed_redirect(state: &AppState, url: &str) -> bool {
    state
        .config
        .allowed_origins
        .iter()
        .any(|origin| url.starts_with(origin))
        || url.starts_with(&state.config.public_url)
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// Retour du fournisseur : échange le code, ouvre la session.
pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> ApiResult<axum::response::Response> {
    if let Some(error) = query.error {
        return Err(ApiError::BadRequest(format!(
            "connexion refusée par le fournisseur : {}",
            error
        )));
    }

    let provider = parse_provider(&provider)?;
    let credentials = credentials(&state, provider)?;

    let code = query
        .code
        .ok_or_else(|| ApiError::BadRequest("code d'autorisation manquant".to_string()))?;
    let csrf_state = query
        .state
        .ok_or_else(|| ApiError::BadRequest("état OAuth manquant".to_string()))?;

    let stored = store::consume_oauth_state(&state.db, &csrf_state)
        .await?
        .ok_or_else(|| ApiError::BadRequest("état OAuth inconnu ou expiré".to_string()))?;

    if stored.provider != provider.as_str() {
        return Err(ApiError::BadRequest(
            "l'état OAuth ne correspond pas au fournisseur".to_string(),
        ));
    }

    let identity = oauth::exchange_code(
        &state.http,
        provider,
        credentials,
        &state.config.redirect_uri(provider.as_str()),
        &code,
    )
    .await
    .map_err(ApiError::Internal)?;

    let user = store::upsert_user_from_identity(&state.db, &identity).await?;

    // Connexion depuis un navigateur pour valider un client sans navigateur :
    // ce dernier récupérera ses jetons par sondage, la page confirme juste.
    if let Some(device_auth_id) = stored.device_auth_id {
        let approved = store::approve_device_request(&state.db, device_auth_id, user.id).await?;
        return Ok(Html(device_result_page(approved, &user.display_name)).into_response());
    }

    let tokens = open_session(&state, user.id, Some("navigateur")).await?;

    // Le site de l'arène récupère les jetons dans le fragment d'URL, qui
    // n'est jamais transmis au serveur ni journalisé.
    if let Some(redirect_to) = stored.redirect_to {
        let separator = if redirect_to.contains('#') { '&' } else { '#' };
        let url = format!(
            "{}{}access_token={}&refresh_token={}&expires_in={}",
            redirect_to,
            separator,
            urlencoding::encode(&tokens.access_token),
            urlencoding::encode(&tokens.refresh_token),
            tokens.expires_in,
        );
        return Ok(Redirect::to(&url).into_response());
    }

    Ok(Json(tokens).into_response())
}

/// Page de confirmation affichée au joueur après appairage d'un appareil.
fn device_result_page(approved: bool, display_name: &str) -> String {
    let (title, message) = if approved {
        (
            "Appareil connecté",
            format!(
                "Bienvenue {} ! Ton appareil est appairé, tu peux revenir dans le jeu.",
                html_escape(display_name)
            ),
        )
    } else {
        (
            "Code expiré",
            "Ce code d'appairage a expiré ou a déjà été utilisé. Relance la connexion depuis le jeu."
                .to_string(),
        )
    };

    format!(
        r#"<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — Monster Battle</title>
<style>
  :root {{ color-scheme: light dark; }}
  body {{
    margin: 0; min-height: 100vh; display: grid; place-items: center;
    font: 16px/1.5 system-ui, sans-serif; background: #12101a; color: #f3f0ff;
  }}
  main {{ max-width: 30rem; padding: 2rem; text-align: center; }}
  h1 {{ font-size: 1.4rem; margin: 0 0 .5rem; }}
  p {{ margin: 0; opacity: .85; }}
  .icon {{ font-size: 3rem; }}
</style>
</head>
<body>
  <main>
    <div class="icon">{icon}</div>
    <h1>{title}</h1>
    <p>{message}</p>
  </main>
</body>
</html>"#,
        title = title,
        message = message,
        icon = if approved { "🎮" } else { "⌛" },
    )
}

/// Échappe le texte inséré dans la page de confirmation : le nom d'affichage
/// vient d'un fournisseur tiers et ne doit pas pouvoir injecter de HTML.
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ── Parcours client sans navigateur ───────────────────────────────────

#[derive(Deserialize, Default)]
pub struct DeviceStartRequest {
    /// Nom lisible de l'appareil, affiché dans la liste des sessions.
    #[serde(default)]
    device_label: Option<String>,
}

#[derive(Serialize)]
pub struct DeviceStartResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

/// Ouvre une demande d'appairage pour un client sans navigateur.
pub async fn device_start(
    State(state): State<AppState>,
    body: Option<Json<DeviceStartRequest>>,
) -> ApiResult<Json<DeviceStartResponse>> {
    let Json(body) = body.unwrap_or_default();

    let request =
        store::create_device_request(&state.db, body.device_label.as_deref(), DEVICE_CODE_TTL)
            .await?;

    Ok(Json(DeviceStartResponse {
        verification_uri: format!(
            "{}/api/v1/auth/device?code={}",
            state.config.public_url,
            urlencoding::encode(&request.user_code)
        ),
        expires_in: (request.expires_at - chrono::Utc::now())
            .num_seconds()
            .max(0) as u64,
        device_code: request.device_code,
        user_code: request.user_code,
        interval: DEVICE_POLL_INTERVAL_SECS,
    }))
}

#[derive(Deserialize)]
pub struct DevicePageQuery {
    code: Option<String>,
}

/// Page ouverte par le joueur pour approuver un appareil : elle propose les
/// fournisseurs configurés, en conservant le code d'appairage.
pub async fn device_page(
    State(state): State<AppState>,
    Query(query): Query<DevicePageQuery>,
) -> Html<String> {
    let code = query.code.unwrap_or_default();

    let mut buttons = String::new();
    for provider in [Provider::GitHub, Provider::Google] {
        let configured = match provider {
            Provider::GitHub => state.config.github.is_some(),
            Provider::Google => state.config.google.is_some(),
        };
        if !configured {
            continue;
        }
        buttons.push_str(&format!(
            r#"<a class="btn" href="/api/v1/auth/{p}/start?user_code={c}">Continuer avec {label}</a>"#,
            p = provider.as_str(),
            c = urlencoding::encode(&code),
            label = if provider == Provider::GitHub { "GitHub" } else { "Google" },
        ));
    }

    if buttons.is_empty() {
        buttons.push_str("<p>Aucun fournisseur de connexion n'est configuré sur ce serveur.</p>");
    }

    Html(format!(
        r#"<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Connecter un appareil — Monster Battle</title>
<style>
  :root {{ color-scheme: light dark; }}
  body {{
    margin: 0; min-height: 100vh; display: grid; place-items: center;
    font: 16px/1.5 system-ui, sans-serif; background: #12101a; color: #f3f0ff;
  }}
  main {{ max-width: 26rem; padding: 2rem; text-align: center; }}
  h1 {{ font-size: 1.4rem; margin: 0 0 .25rem; }}
  code {{
    display: inline-block; margin: 1rem 0; padding: .5rem 1rem;
    font-size: 1.5rem; letter-spacing: .15em;
    background: #241f38; border-radius: .5rem;
  }}
  .btn {{
    display: block; margin: .5rem 0; padding: .75rem 1rem;
    background: #6d5ce7; color: #fff; border-radius: .5rem;
    text-decoration: none; font-weight: 600;
  }}
  .btn:hover {{ background: #7f70ec; }}
</style>
</head>
<body>
  <main>
    <div style="font-size:3rem">🎮</div>
    <h1>Connecter un appareil</h1>
    <p>Code affiché par le jeu :</p>
    <code>{code}</code>
    {buttons}
  </main>
</body>
</html>"#,
        code = html_escape(&code),
        buttons = buttons,
    ))
}

#[derive(Deserialize)]
pub struct DeviceTokenRequest {
    device_code: String,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceTokenResponse {
    /// Le joueur n'a pas encore approuvé : réessayer après `interval`.
    Pending { interval: u64 },
    /// Session ouverte.
    Ready(SessionTokens),
}

/// Sondage du client : le joueur a-t-il approuvé ?
pub async fn device_token(
    State(state): State<AppState>,
    Json(body): Json<DeviceTokenRequest>,
) -> ApiResult<Json<DeviceTokenResponse>> {
    match store::poll_device_request(&state.db, &body.device_code).await? {
        DevicePollOutcome::Pending => Ok(Json(DeviceTokenResponse::Pending {
            interval: DEVICE_POLL_INTERVAL_SECS,
        })),
        DevicePollOutcome::Approved(user_id) => {
            let tokens = open_session(&state, user_id, Some("appareil appairé")).await?;
            Ok(Json(DeviceTokenResponse::Ready(tokens)))
        }
        DevicePollOutcome::Expired => Err(ApiError::BadRequest(
            "demande d'appairage expirée ou déjà utilisée".to_string(),
        )),
    }
}

#[derive(Deserialize)]
pub struct DeviceApproveRequest {
    user_code: String,
}

/// Approuve un appairage depuis une session déjà connectée (le site de
/// l'arène, typiquement, où le joueur saisit le code affiché par le jeu).
pub async fn device_approve(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<DeviceApproveRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let request_id = store::find_pending_device_request(&state.db, &body.user_code)
        .await?
        .ok_or_else(|| ApiError::BadRequest("code inconnu ou expiré".to_string()))?;

    let approved = store::approve_device_request(&state.db, request_id, user_id).await?;
    if !approved {
        return Err(ApiError::BadRequest(
            "ce code a déjà été utilisé".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({ "approved": true })))
}

// ── Session ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

/// Renouvelle une session. Le refresh token est tourné à chaque appel.
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> ApiResult<Json<SessionTokens>> {
    let rotated = store::rotate_refresh_token(
        &state.db,
        &body.refresh_token,
        state.config.refresh_token_ttl,
    )
    .await?;

    let (user_id, refresh_token) = rotated.ok_or(ApiError::Unauthorized)?;

    let user = store::find_user(&state.db, user_id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let access_token = jwt::encode(
        &state.config.jwt_secret,
        user_id,
        state.config.access_token_ttl,
    )
    .map_err(ApiError::Internal)?;

    Ok(Json(SessionTokens {
        access_token,
        refresh_token,
        expires_in: state.config.access_token_ttl.as_secs(),
        user_id,
        display_name: user.display_name,
    }))
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    refresh_token: String,
}

/// Déconnecte un appareil en révoquant son refresh token.
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let revoked = store::revoke_refresh_token(&state.db, &body.refresh_token).await?;
    Ok(Json(serde_json::json!({ "revoked": revoked })))
}

#[derive(Serialize)]
pub struct MeResponse {
    id: uuid::Uuid,
    display_name: String,
    avatar_url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    monsters_alive: i64,
}

/// Profil du joueur connecté.
pub async fn me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> ApiResult<Json<MeResponse>> {
    let user = store::find_user(&state.db, user_id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let monsters_alive = crate::monsters::store::count_alive(&state.db, user_id).await?;

    Ok(Json(MeResponse {
        id: user.id,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        created_at: user.created_at,
        monsters_alive,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_nom_d_affichage_est_echappe_dans_la_page() {
        let page = device_result_page(true, "<script>alert(1)</script>");
        assert!(!page.contains("<script>alert"));
        assert!(page.contains("&lt;script&gt;"));
    }

    #[test]
    fn la_page_d_echec_ne_reprend_pas_le_nom() {
        let page = device_result_page(false, "Peu importe");
        assert!(!page.contains("Peu importe"));
        assert!(page.contains("expiré"));
    }
}
