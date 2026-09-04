//! Configuration du serveur, lue depuis l'environnement.

use std::time::Duration;

/// Durée de validité d'un jeton d'accès (court : il n'est pas révocable).
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(60 * 60);

/// Durée de validité d'un jeton de rafraîchissement (long : révocable en base).
const REFRESH_TOKEN_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 60);

/// Identifiants d'une application OAuth chez un fournisseur.
#[derive(Debug, Clone)]
pub struct OAuthCredentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Configuration complète du serveur.
#[derive(Debug, Clone)]
pub struct Config {
    /// Port d'écoute (WebSocket de combat + API HTTP sur le même port).
    pub port: u16,

    /// URL de la base Postgres. `None` = mode relais seul, sans comptes.
    pub database_url: Option<String>,

    /// URL publique du serveur, utilisée pour construire les URLs de callback
    /// OAuth et l'adresse d'appairage montrée dans le terminal.
    pub public_url: String,

    /// Secret de signature des jetons d'accès (HS256).
    pub jwt_secret: String,

    /// Origines autorisées pour le CORS (site de l'arène). Vide = tout autoriser.
    pub allowed_origins: Vec<String>,

    pub github: Option<OAuthCredentials>,
    pub google: Option<OAuthCredentials>,

    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
}

impl Config {
    /// Lit la configuration depuis l'environnement.
    ///
    /// Seul `PORT` a une valeur par défaut : sans `DATABASE_URL` le serveur
    /// démarre en mode relais seul (comportement historique), et sans
    /// `JWT_SECRET` l'API refuse de démarrer plutôt que d'utiliser un secret
    /// deviné.
    pub fn from_env() -> anyhow::Result<Self> {
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "7878".to_string())
            .parse()
            .map_err(|_| anyhow::anyhow!("PORT n'est pas un numéro de port valide"))?;

        let database_url = non_empty("DATABASE_URL");

        let public_url = non_empty("PUBLIC_URL")
            .unwrap_or_else(|| format!("http://localhost:{}", port))
            .trim_end_matches('/')
            .to_string();

        let jwt_secret = match non_empty("JWT_SECRET") {
            Some(secret) => secret,
            None if database_url.is_none() => String::new(),
            None => anyhow::bail!("JWT_SECRET est obligatoire quand DATABASE_URL est défini"),
        };

        if database_url.is_some() && jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET doit faire au moins 32 caractères");
        }

        let allowed_origins = non_empty("ALLOWED_ORIGINS")
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            port,
            database_url,
            public_url,
            jwt_secret,
            allowed_origins,
            github: credentials("GITHUB"),
            google: credentials("GOOGLE"),
            access_token_ttl: ACCESS_TOKEN_TTL,
            refresh_token_ttl: REFRESH_TOKEN_TTL,
        })
    }

    /// URL de callback OAuth pour un fournisseur donné.
    pub fn redirect_uri(&self, provider: &str) -> String {
        format!("{}/api/v1/auth/{}/callback", self.public_url, provider)
    }
}

/// Lit une variable d'environnement en traitant la chaîne vide comme absente.
fn non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// Lit `{PREFIX}_CLIENT_ID` / `{PREFIX}_CLIENT_SECRET`.
fn credentials(prefix: &str) -> Option<OAuthCredentials> {
    Some(OAuthCredentials {
        client_id: non_empty(&format!("{}_CLIENT_ID", prefix))?,
        client_secret: non_empty(&format!("{}_CLIENT_SECRET", prefix))?,
    })
}
