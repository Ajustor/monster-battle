//! Jetons d'accès JWT (HS256).
//!
//! Le jeton d'accès est volontairement court et non révocable : c'est le
//! refresh token, stocké haché en base, qui porte la session révocable.

use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Contenu d'un jeton d'accès.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Identifiant du joueur.
    pub sub: Uuid,
    /// Date d'émission (epoch secondes).
    pub iat: i64,
    /// Date d'expiration (epoch secondes).
    pub exp: i64,
}

/// Encode un jeton d'accès pour un joueur.
pub fn encode(secret: &str, user_id: Uuid, ttl: Duration) -> anyhow::Result<String> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        iat: now,
        exp: now + ttl.as_secs() as i64,
    };
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// Tolérance d'horloge acceptée sur l'expiration.
///
/// Les clients (terminal, mobile, navigateur) n'ont pas forcément une horloge
/// parfaitement réglée : sans cette marge, un décalage de quelques secondes
/// suffirait à faire échouer des requêtes légitimes juste avant le
/// rafraîchissement du jeton.
const CLOCK_SKEW_LEEWAY_SECS: u64 = 60;

/// Décode et valide un jeton d'accès. Retourne `None` s'il est invalide ou expiré.
pub fn decode(secret: &str, token: &str) -> Option<Claims> {
    let mut validation = Validation::default();
    validation.leeway = CLOCK_SKEW_LEEWAY_SECS;

    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "un-secret-de-test-suffisamment-long-1234";

    #[test]
    fn aller_retour_encode_decode() {
        let id = Uuid::new_v4();
        let token = encode(SECRET, id, Duration::from_secs(60)).unwrap();
        let claims = decode(SECRET, &token).expect("jeton valide");
        assert_eq!(claims.sub, id);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn un_autre_secret_est_rejete() {
        let token = encode(SECRET, Uuid::new_v4(), Duration::from_secs(60)).unwrap();
        assert!(decode("un-autre-secret-tout-aussi-long-000000", &token).is_none());
    }

    /// Forge un jeton dont l'expiration est dans le passé, ce que l'API
    /// publique ne permet pas (sa durée de vie est non signée).
    fn expired_token(seconds_ago: i64) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: Uuid::new_v4(),
            iat: now - seconds_ago - 60,
            exp: now - seconds_ago,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn un_jeton_expire_est_rejete() {
        assert!(decode(SECRET, &expired_token(CLOCK_SKEW_LEEWAY_SECS as i64 + 10)).is_none());
    }

    #[test]
    fn la_marge_d_horloge_est_toleree() {
        // Tout juste expiré : accepté, le client a le temps de rafraîchir.
        assert!(decode(SECRET, &expired_token(5)).is_some());
    }

    #[test]
    fn un_jeton_malforme_est_rejete() {
        assert!(decode(SECRET, "pas-du-tout-un-jwt").is_none());
    }
}
