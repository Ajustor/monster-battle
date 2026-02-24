use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Taille du nonce AES-GCM (96 bits).
const NONCE_SIZE: usize = 12;

/// Dérive une clé AES-256 à partir d'un secret (machine ID + salt).
pub fn derive_key(secret: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"monster-battle-v1-");
    hasher.update(secret);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Chiffre des données avec AES-256-GCM.
/// Retourne : nonce (12 bytes) || ciphertext.
pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Clé invalide: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Erreur de chiffrement: {}", e))?;

    // Préfixe le nonce au ciphertext
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Déchiffre des données chiffrées avec AES-256-GCM.
/// Attend : nonce (12 bytes) || ciphertext.
pub fn decrypt(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if encrypted.len() < NONCE_SIZE {
        return Err("Données chiffrées trop courtes".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Clé invalide: {}", e))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Erreur de déchiffrement (fichier modifié ?): {}", e))
}

/// Génère un secret unique basé sur la machine.
/// Utilise le hostname + un sel fixe pour que ce soit déterministe par machine.
pub fn machine_secret() -> Vec<u8> {
    let hostname = std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string());

    let mut hasher = Sha256::new();
    hasher.update(b"monster-battle-machine-");
    hasher.update(hostname.as_bytes());

    // Ajoute le username si disponible
    if let Ok(user) = std::env::var("USER") {
        hasher.update(user.as_bytes());
    }

    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key(b"test-secret");
        let data = b"Hello, Monster Battle!";

        let encrypted = encrypt(data, &key).unwrap();
        assert_ne!(&encrypted, data); // Should be different

        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_key(b"secret-1");
        let key2 = derive_key(b"secret-2");
        let data = b"Sensitive data";

        let encrypted = encrypt(data, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_data_fails() {
        let key = derive_key(b"test-secret");
        let data = b"Original data";

        let mut encrypted = encrypt(data, &key).unwrap();
        // Tamper with the ciphertext
        if let Some(last) = encrypted.last_mut() {
            *last ^= 0xFF;
        }

        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }
}
