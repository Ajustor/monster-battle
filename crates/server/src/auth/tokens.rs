//! Génération des secrets opaques (refresh tokens, device codes, états CSRF)
//! et de leur empreinte de stockage.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Alphabet des codes utilisateur : ni 0/O ni 1/I/L, pour être dictable à voix
/// haute et recopiable sans ambiguïté depuis un terminal.
const USER_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Nombre de caractères d'un code utilisateur (hors tiret).
const USER_CODE_LEN: usize = 8;

/// Génère un secret aléatoire de 256 bits encodé en base64url.
pub fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Empreinte SHA-256 d'un secret, telle que stockée en base.
///
/// Les secrets ont 256 bits d'entropie : un simple SHA-256 suffit, une KDF
/// lente (argon2) ne servirait qu'à des secrets devinables comme un mot de passe.
pub fn hash_secret(secret: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().to_vec()
}

/// Génère un code utilisateur lisible du type `WXYZ-2345`.
pub fn random_user_code() -> String {
    let mut bytes = [0u8; USER_CODE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut bytes);

    let mut code = String::with_capacity(USER_CODE_LEN + 1);
    for (i, b) in bytes.iter().enumerate() {
        if i == USER_CODE_LEN / 2 {
            code.push('-');
        }
        code.push(USER_CODE_ALPHABET[*b as usize % USER_CODE_ALPHABET.len()] as char);
    }
    code
}

/// Normalise un code saisi par un humain : majuscules, sans espaces ni tirets.
pub fn normalize_user_code(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn les_secrets_sont_uniques_et_longs() {
        let secrets: HashSet<String> = (0..100).map(|_| random_secret()).collect();
        assert_eq!(secrets.len(), 100, "collision sur 100 tirages");
        // 32 octets en base64url sans padding = 43 caractères.
        assert!(secrets.iter().all(|s| s.len() == 43));
    }

    #[test]
    fn le_hash_est_stable_et_discriminant() {
        assert_eq!(hash_secret("abc"), hash_secret("abc"));
        assert_ne!(hash_secret("abc"), hash_secret("abd"));
        assert_eq!(hash_secret("abc").len(), 32);
    }

    #[test]
    fn le_code_utilisateur_est_lisible() {
        for _ in 0..50 {
            let code = random_user_code();
            assert_eq!(code.len(), USER_CODE_LEN + 1);
            assert_eq!(code.chars().nth(4), Some('-'));
            // Aucun caractère ambigu : ni 0/O, ni 1/I/L.
            assert!(
                !code.contains(['0', 'O', '1', 'I', 'L']),
                "code ambigu : {}",
                code
            );
        }
    }

    #[test]
    fn la_normalisation_accepte_les_saisies_humaines() {
        assert_eq!(normalize_user_code("wxyz-2345"), "WXYZ2345");
        assert_eq!(normalize_user_code(" WXYZ 2345 "), "WXYZ2345");
        assert_eq!(normalize_user_code("WXYZ2345"), "WXYZ2345");
    }

    #[test]
    fn un_code_genere_survit_a_sa_normalisation() {
        let code = random_user_code();
        assert_eq!(normalize_user_code(&code), code.replace('-', ""));
    }
}
