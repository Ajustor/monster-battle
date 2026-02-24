mod crypto;
mod local;

pub use crypto::{decrypt, derive_key, encrypt, machine_secret};
pub use local::LocalStorage;

use monster_battle_core::Monster;
use uuid::Uuid;

/// Erreurs possibles du stockage.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Monstre introuvable : {0}")]
    NotFound(Uuid),

    #[error("Erreur d'entrée/sortie : {0}")]
    Io(#[from] std::io::Error),

    #[error("Erreur de sérialisation : {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Erreur de chiffrement : {0}")]
    Encryption(String),
}

/// Trait définissant les opérations de stockage des monstres.
pub trait MonsterStorage {
    /// Sauvegarde un monstre (crée ou met à jour).
    fn save(&self, monster: &Monster) -> Result<(), StorageError>;

    /// Charge un monstre par son UUID.
    fn load(&self, id: Uuid) -> Result<Monster, StorageError>;

    /// Supprime un monstre par son UUID.
    fn delete(&self, id: Uuid) -> Result<(), StorageError>;

    /// Liste tous les monstres sauvegardés.
    fn list_all(&self) -> Result<Vec<Monster>, StorageError>;

    /// Liste uniquement les monstres vivants.
    fn list_alive(&self) -> Result<Vec<Monster>, StorageError> {
        Ok(self
            .list_all()?
            .into_iter()
            .filter(|m| m.is_alive())
            .collect())
    }

    /// Liste uniquement les monstres morts (cimetière).
    fn list_dead(&self) -> Result<Vec<Monster>, StorageError> {
        Ok(self
            .list_all()?
            .into_iter()
            .filter(|m| m.is_dead())
            .collect())
    }

    /// Exporte un monstre en JSON pour l'échange réseau (non chiffré).
    fn export_for_network(&self, id: Uuid) -> Result<String, StorageError> {
        let monster = self.load(id)?;
        serde_json::to_string(&monster).map_err(StorageError::Serialization)
    }

    /// Importe un monstre depuis du JSON réseau.
    fn import_from_network(&self, json: &str) -> Result<Monster, StorageError> {
        let monster: Monster = serde_json::from_str(json).map_err(StorageError::Serialization)?;
        Ok(monster)
    }
}
