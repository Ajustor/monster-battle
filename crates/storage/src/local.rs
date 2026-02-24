use std::fs;
use std::path::{Path, PathBuf};

use monster_battle_core::Monster;
use uuid::Uuid;

use crate::crypto;
use crate::{MonsterStorage, StorageError};

/// Stockage local sur le système de fichiers.
/// - Les monstres **vivants** sont chiffrés (AES-256-GCM) → fichiers `.enc`
/// - Les monstres **morts** sont en JSON clair → fichiers `.json` (lecture seule, souvenir)
pub struct LocalStorage {
    base_dir: PathBuf,
    /// Clé de chiffrement dérivée de la machine.
    encryption_key: [u8; 32],
}

impl LocalStorage {
    /// Crée un nouveau stockage local dans le répertoire spécifié.
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self, StorageError> {
        let base_dir = base_dir.as_ref().to_path_buf();

        let alive_dir = base_dir.join("alive");
        let dead_dir = base_dir.join("dead");

        fs::create_dir_all(&alive_dir)?;
        fs::create_dir_all(&dead_dir)?;

        // Dérive la clé de chiffrement à partir du secret machine
        let secret = crypto::machine_secret();
        let encryption_key = crypto::derive_key(&secret);

        Ok(Self {
            base_dir,
            encryption_key,
        })
    }

    /// Chemin pour un monstre vivant (chiffré).
    fn alive_path(&self, id: Uuid) -> PathBuf {
        self.base_dir.join("alive").join(format!("{}.enc", id))
    }

    /// Chemin pour un monstre mort (JSON clair).
    fn dead_path(&self, id: Uuid) -> PathBuf {
        self.base_dir.join("dead").join(format!("{}.json", id))
    }

    /// Cherche un monstre dans les deux répertoires.
    fn find_monster_path(&self, id: Uuid) -> Option<PathBuf> {
        let alive = self.alive_path(id);
        if alive.exists() {
            return Some(alive);
        }
        // Ancien format non chiffré (migration)
        let alive_json = self.base_dir.join("alive").join(format!("{}.json", id));
        if alive_json.exists() {
            return Some(alive_json);
        }
        let dead = self.dead_path(id);
        if dead.exists() {
            return Some(dead);
        }
        None
    }

    /// Sauvegarde un monstre vivant (chiffré).
    fn save_alive(&self, monster: &Monster) -> Result<(), StorageError> {
        let json = serde_json::to_string(monster)?;
        let encrypted = crypto::encrypt(json.as_bytes(), &self.encryption_key)
            .map_err(StorageError::Encryption)?;
        let path = self.alive_path(monster.id);
        fs::write(path, encrypted)?;
        Ok(())
    }

    /// Sauvegarde un monstre mort (JSON clair, en souvenir).
    fn save_dead(&self, monster: &Monster) -> Result<(), StorageError> {
        let json = serde_json::to_string_pretty(monster)?;
        let path = self.dead_path(monster.id);
        fs::write(path, json)?;
        Ok(())
    }

    /// Charge un monstre depuis un fichier (détecte chiffré vs JSON).
    fn load_from_path(&self, path: &Path) -> Result<Monster, StorageError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "enc" => {
                let encrypted = fs::read(path)?;
                let decrypted = crypto::decrypt(&encrypted, &self.encryption_key)
                    .map_err(StorageError::Encryption)?;
                let json = String::from_utf8(decrypted)
                    .map_err(|e| StorageError::Encryption(format!("UTF-8 invalide: {}", e)))?;
                let monster: Monster = serde_json::from_str(&json)?;
                Ok(monster)
            }
            "json" => {
                let data = fs::read_to_string(path)?;
                let monster: Monster = serde_json::from_str(&data)?;
                Ok(monster)
            }
            _ => Err(StorageError::Encryption(format!(
                "Extension inconnue: {}",
                ext
            ))),
        }
    }

    /// Charge tous les monstres d'un sous-répertoire.
    fn load_dir(&self, subdir: &str) -> Result<Vec<Monster>, StorageError> {
        let dir = self.base_dir.join(subdir);
        let mut monsters = Vec::new();

        if !dir.exists() {
            return Ok(monsters);
        }

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if ext == "enc" || ext == "json" {
                match self.load_from_path(&path) {
                    Ok(monster) => monsters.push(monster),
                    Err(e) => {
                        eprintln!(
                            "Avertissement : impossible de charger {} : {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(monsters)
    }

    /// Migre un ancien fichier JSON non chiffré vers le format chiffré.
    fn migrate_if_needed(&self, id: Uuid) {
        let old_json = self.base_dir.join("alive").join(format!("{}.json", id));
        if old_json.exists() {
            if let Ok(data) = fs::read_to_string(&old_json) {
                if let Ok(monster) = serde_json::from_str::<Monster>(&data) {
                    if self.save_alive(&monster).is_ok() {
                        let _ = fs::remove_file(&old_json);
                    }
                }
            }
        }
    }
}

impl MonsterStorage for LocalStorage {
    fn save(&self, monster: &Monster) -> Result<(), StorageError> {
        // Supprime l'ancien fichier (peut avoir changé de répertoire/format)
        if let Some(old_path) = self.find_monster_path(monster.id) {
            let _ = fs::remove_file(old_path);
        }

        if monster.is_dead() {
            self.save_dead(monster)
        } else {
            self.save_alive(monster)
        }
    }

    fn load(&self, id: Uuid) -> Result<Monster, StorageError> {
        // Tente une migration automatique si nécessaire
        self.migrate_if_needed(id);

        let path = self
            .find_monster_path(id)
            .ok_or(StorageError::NotFound(id))?;

        self.load_from_path(&path)
    }

    fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        let path = self
            .find_monster_path(id)
            .ok_or(StorageError::NotFound(id))?;

        fs::remove_file(path)?;
        Ok(())
    }

    fn list_all(&self) -> Result<Vec<Monster>, StorageError> {
        let mut monsters = self.load_dir("alive")?;
        monsters.extend(self.load_dir("dead")?);
        Ok(monsters)
    }

    fn list_alive(&self) -> Result<Vec<Monster>, StorageError> {
        self.load_dir("alive")
    }

    fn list_dead(&self) -> Result<Vec<Monster>, StorageError> {
        self.load_dir("dead")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_battle_core::Monster;
    use monster_battle_core::genetics::generate_starter_stats;
    use monster_battle_core::types::ElementType;

    #[test]
    fn test_save_and_load_encrypted() {
        let dir = std::env::temp_dir().join("monster_test_encrypted");
        let _ = fs::remove_dir_all(&dir);
        let storage = LocalStorage::new(&dir).unwrap();

        let monster = Monster::new_starter(
            "CryptoMon".to_string(),
            ElementType::Fire,
            generate_starter_stats(ElementType::Fire),
        );
        let id = monster.id;

        storage.save(&monster).unwrap();

        // Le fichier doit être .enc, pas .json
        let enc_path = storage.alive_path(id);
        assert!(enc_path.exists(), "Fichier .enc doit exister");

        // Le contenu ne doit PAS être du JSON lisible
        let raw = fs::read(&enc_path).unwrap();
        assert!(
            serde_json::from_slice::<Monster>(&raw).is_err(),
            "Le fichier chiffré ne doit pas être du JSON valide"
        );

        // Mais on peut le charger correctement
        let loaded = storage.load(id).unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.name, "CryptoMon");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_dead_monster_saved_as_json() {
        let dir = std::env::temp_dir().join("monster_test_dead_json");
        let _ = fs::remove_dir_all(&dir);
        let storage = LocalStorage::new(&dir).unwrap();

        let mut monster = Monster::new_starter(
            "DeadMon".to_string(),
            ElementType::Shadow,
            generate_starter_stats(ElementType::Shadow),
        );
        monster.died_at = Some(chrono::Utc::now());
        let id = monster.id;

        storage.save(&monster).unwrap();

        // Le fichier doit être .json (mort = lisible)
        let json_path = storage.dead_path(id);
        assert!(
            json_path.exists(),
            "Fichier .json doit exister pour les morts"
        );

        // Le contenu doit être du JSON lisible
        let data = fs::read_to_string(&json_path).unwrap();
        let parsed: Monster = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.name, "DeadMon");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_all() {
        let dir = std::env::temp_dir().join("monster_test_list_enc");
        let _ = fs::remove_dir_all(&dir);
        let storage = LocalStorage::new(&dir).unwrap();

        let m1 = Monster::new_starter(
            "A".to_string(),
            ElementType::Fire,
            generate_starter_stats(ElementType::Fire),
        );
        let m2 = Monster::new_starter(
            "B".to_string(),
            ElementType::Water,
            generate_starter_stats(ElementType::Water),
        );

        storage.save(&m1).unwrap();
        storage.save(&m2).unwrap();

        let all = storage.list_all().unwrap();
        assert_eq!(all.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }
}
