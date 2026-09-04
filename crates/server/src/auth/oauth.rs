//! Fournisseurs OAuth 2.0 (GitHub, Google).
//!
//! On ne dépend pas d'un crate OAuth générique : le flux « authorization
//! code » se résume à construire une URL d'autorisation puis à échanger le
//! code contre un profil, ce que ces deux fournisseurs font de façon assez
//! différente pour qu'un traitement explicite soit plus lisible.

use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

use crate::config::OAuthCredentials;

/// Fournisseur d'identité supporté.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    GitHub,
    Google,
}

impl Provider {
    /// Nom court utilisé dans les URLs et stocké en base.
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::GitHub => "github",
            Provider::Google => "google",
        }
    }

    /// Point d'entrée du consentement utilisateur.
    fn authorize_url(&self) -> &'static str {
        match self {
            Provider::GitHub => "https://github.com/login/oauth/authorize",
            Provider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    /// Point d'échange code → jeton d'accès.
    fn token_url(&self) -> &'static str {
        match self {
            Provider::GitHub => "https://github.com/login/oauth/access_token",
            Provider::Google => "https://oauth2.googleapis.com/token",
        }
    }

    /// Portées demandées : le strict minimum pour identifier le joueur.
    fn scopes(&self) -> &'static str {
        match self {
            Provider::GitHub => "read:user user:email",
            Provider::Google => "openid email profile",
        }
    }

    /// Construit l'URL vers laquelle rediriger le navigateur du joueur.
    pub fn authorization_url(
        &self,
        credentials: &OAuthCredentials,
        redirect_uri: &str,
        state: &str,
    ) -> String {
        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&state={}&scope={}&response_type=code",
            self.authorize_url(),
            urlencoding::encode(&credentials.client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state),
            urlencoding::encode(self.scopes()),
        );
        if *self == Provider::Google {
            // Sans cela Google ne redemande jamais le consentement et ne
            // renvoie pas l'email sur les reconnexions.
            url.push_str("&access_type=online&prompt=select_account");
        }
        url
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Provider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "github" => Ok(Provider::GitHub),
            "google" => Ok(Provider::Google),
            _ => Err(()),
        }
    }
}

/// Profil minimal récupéré chez le fournisseur.
#[derive(Debug, Clone)]
pub struct ProviderIdentity {
    pub provider: Provider,
    /// Identifiant stable du compte chez le fournisseur.
    pub provider_user_id: String,
    pub email: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Deserialize)]
struct GoogleUser {
    sub: String,
    name: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    picture: Option<String>,
}

/// Échange un code d'autorisation contre le profil du joueur.
pub async fn exchange_code(
    http: &reqwest::Client,
    provider: Provider,
    credentials: &OAuthCredentials,
    redirect_uri: &str,
    code: &str,
) -> anyhow::Result<ProviderIdentity> {
    let params = [
        ("client_id", credentials.client_id.as_str()),
        ("client_secret", credentials.client_secret.as_str()),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let response = http
        .post(provider.token_url())
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "{} a refusé l'échange du code (HTTP {})",
            provider,
            response.status()
        );
    }

    let token: TokenResponse = response.json().await?;

    match provider {
        Provider::GitHub => fetch_github_identity(http, &token.access_token).await,
        Provider::Google => fetch_google_identity(http, &token.access_token).await,
    }
}

async fn fetch_github_identity(
    http: &reqwest::Client,
    access_token: &str,
) -> anyhow::Result<ProviderIdentity> {
    let user: GitHubUser = http
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // L'email du profil est absent quand le joueur le garde privé : on
    // interroge alors la liste des emails vérifiés.
    let email = match user.email {
        Some(email) => Some(email),
        None => {
            let emails: Vec<GitHubEmail> = http
                .get("https://api.github.com/user/emails")
                .bearer_auth(access_token)
                .header("Accept", "application/vnd.github+json")
                .send()
                .await?
                .error_for_status()?
                .json()
                .await
                .unwrap_or_default();

            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
        }
    };

    Ok(ProviderIdentity {
        provider: Provider::GitHub,
        provider_user_id: user.id.to_string(),
        display_name: user.name.unwrap_or_else(|| user.login.clone()),
        email,
        avatar_url: user.avatar_url,
    })
}

async fn fetch_google_identity(
    http: &reqwest::Client,
    access_token: &str,
) -> anyhow::Result<ProviderIdentity> {
    let user: GoogleUser = http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Un email non vérifié ne doit pas servir à rattacher une identité à un
    // compte existant : on l'ignore purement et simplement.
    let email = user.email.filter(|_| user.email_verified.unwrap_or(false));

    Ok(ProviderIdentity {
        provider: Provider::Google,
        display_name: user
            .name
            .or_else(|| email.clone())
            .unwrap_or_else(|| format!("Dresseur {}", &user.sub[..6.min(user.sub.len())])),
        provider_user_id: user.sub,
        email,
        avatar_url: user.picture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials() -> OAuthCredentials {
        OAuthCredentials {
            client_id: "id client/avec des caractères".to_string(),
            client_secret: "secret".to_string(),
        }
    }

    #[test]
    fn les_noms_de_fournisseurs_font_un_aller_retour() {
        for provider in [Provider::GitHub, Provider::Google] {
            assert_eq!(Provider::from_str(provider.as_str()), Ok(provider));
        }
        assert_eq!(Provider::from_str("GitHub"), Ok(Provider::GitHub));
        assert!(Provider::from_str("facebook").is_err());
    }

    #[test]
    fn l_url_d_autorisation_encode_ses_parametres() {
        let url = Provider::GitHub.authorization_url(
            &credentials(),
            "https://arene.example/api/v1/auth/github/callback",
            "un état/avec des caractères",
        );
        assert!(url.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(url.contains("client_id=id%20client%2Favec%20des%20caract%C3%A8res"));
        assert!(url.contains("state=un%20%C3%A9tat%2Favec%20des%20caract%C3%A8res"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Farene.example%2F"));
        // Aucun paramètre brut ne doit casser la query string.
        assert!(!url.contains(' '));
    }

    #[test]
    fn google_force_la_selection_de_compte() {
        let url = Provider::Google.authorization_url(&credentials(), "https://x/cb", "s");
        assert!(url.contains("prompt=select_account"));

        let url = Provider::GitHub.authorization_url(&credentials(), "https://x/cb", "s");
        assert!(!url.contains("prompt="));
    }
}
