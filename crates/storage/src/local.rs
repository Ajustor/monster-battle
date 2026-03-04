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
    /// Seuls les fichiers `.enc` sont acceptés dans `alive/` — les fichiers
    /// `.json` dans `alive/` ne sont plus acceptés (anti-triche : empêche de
    /// copier un fichier mort dans le dossier des vivants).
    fn find_monster_path(&self, id: Uuid) -> Option<PathBuf> {
        let alive = self.alive_path(id);
        if alive.exists() {
            return Some(alive);
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
    /// Pour le dossier `alive`, seuls les fichiers `.enc` sont acceptés (anti-triche).
    /// Pour le dossier `dead`, seuls les fichiers `.json` sont acceptés.
    fn load_dir(&self, subdir: &str) -> Result<Vec<Monster>, StorageError> {
        let dir = self.base_dir.join(subdir);
        let mut monsters = Vec::new();

        if !dir.exists() {
            return Ok(monsters);
        }

        let is_alive_dir = subdir == "alive";
        let expected_ext = if is_alive_dir { "enc" } else { "json" };

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            // N'accepter que l'extension attendue pour ce répertoire
            if ext != expected_ext {
                // Supprimer les fichiers .json dans alive/ (anti-triche / nettoyage)
                if is_alive_dir && ext == "json" {
                    eprintln!(
                        "Anti-triche : fichier JSON ignoré et supprimé dans alive/ : {}",
                        path.display()
                    );
                    let _ = fs::remove_file(&path);
                }
                continue;
            }

            match self.load_from_path(&path) {
                Ok(monster) => {
                    // Vérification d'intégrité : un monstre dans alive/ doit être vivant
                    if is_alive_dir && monster.is_dead() {
                        eprintln!(
                            "Anti-triche : monstre mort trouvé dans alive/, déplacement vers dead/ : {}",
                            monster.name
                        );
                        // Déplacer vers dead/
                        let _ = fs::remove_file(&path);
                        let _ = self.save_dead(&monster);
                        continue;
                    }
                    // Vérification inverse : un monstre dans dead/ doit être mort
                    if !is_alive_dir && monster.is_alive() {
                        eprintln!(
                            "Incohérence : monstre vivant trouvé dans dead/, ignoré : {}",
                            monster.name
                        );
                        continue;
                    }
                    monsters.push(monster);
                }
                Err(e) => {
                    eprintln!(
                        "Avertissement : impossible de charger {} : {}",
                        path.display(),
                        e
                    );
                    // Si le fichier chiffré est corrompu dans alive/, le supprimer
                    if is_alive_dir {
                        eprintln!(
                            "Anti-triche : fichier corrompu supprimé dans alive/ : {}",
                            path.display()
                        );
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }

        Ok(monsters)
    }
}

impl MonsterStorage for LocalStorage {
    fn save(&self, monster: &Monster) -> Result<(), StorageError> {
        // Anti-triche : empêcher la résurrection d'un monstre mort.
        // Si le monstre existe déjà dans dead/ et qu'on essaie de le sauvegarder
        // comme vivant, c'est une tentative de triche.
        let dead_path = self.dead_path(monster.id);
        if monster.is_alive() && dead_path.exists() {
            eprintln!(
                "Anti-triche : tentative de résurrection bloquée pour {} ({})",
                monster.name, monster.id
            );
            return Err(StorageError::Encryption(
                "Impossible de ressusciter un monstre mort.".to_string(),
            ));
        }

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
        let path = self
            .find_monster_path(id)
            .ok_or(StorageError::NotFound(id))?;

        let monster = self.load_from_path(&path)?;

        // Vérification d'intégrité : si un monstre du dossier alive/ est mort,
        // le déplacer vers dead/
        if path.starts_with(self.base_dir.join("alive")) && monster.is_dead() {
            let _ = fs::remove_file(&path);
            self.save_dead(&monster)?;
        }

        Ok(monster)
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

    #[test]
    fn test_anticheat_json_in_alive_is_rejected() {
        // Simuler la triche : copier un fichier JSON dans alive/
        let dir = std::env::temp_dir().join("monster_test_anticheat_json");
        let _ = fs::remove_dir_all(&dir);
        let storage = LocalStorage::new(&dir).unwrap();

        let monster = Monster::new_starter(
            "TricheMon".to_string(),
            ElementType::Fire,
            generate_starter_stats(ElementType::Fire),
        );
        let id = monster.id;

        // Écrire un fichier JSON directement dans alive/ (triche)
        let json = serde_json::to_string(&monster).unwrap();
        let cheat_path = dir.join("alive").join(format!("{}.json", id));
        fs::write(&cheat_path, json).unwrap();

        // Le fichier JSON doit être supprimé et le monstre ignoré
        let alive = storage.list_alive().unwrap();
        assert!(
            alive.is_empty(),
            "Le monstre JSON en alive/ doit être ignoré"
        );
        assert!(
            !cheat_path.exists(),
            "Le fichier JSON triche doit être supprimé"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_anticheat_resurrection_blocked() {
        // Simuler la triche : essayer de sauvegarder un monstre mort comme vivant
        let dir = std::env::temp_dir().join("monster_test_anticheat_resurrect");
        let _ = fs::remove_dir_all(&dir);
        let storage = LocalStorage::new(&dir).unwrap();

        let mut monster = Monster::new_starter(
            "MortMon".to_string(),
            ElementType::Shadow,
            generate_starter_stats(ElementType::Shadow),
        );
        let id = monster.id;

        // Tuer et sauvegarder le monstre
        monster.died_at = Some(chrono::Utc::now());
        storage.save(&monster).unwrap();

        // Vérifier qu'il est dans dead/
        let dead = storage.list_dead().unwrap();
        assert_eq!(dead.len(), 1);

        // Tentative de résurrection : remettre died_at à None
        monster.died_at = None;
        let result = storage.save(&monster);
        assert!(result.is_err(), "La résurrection doit être bloquée");

        // Le monstre doit toujours être dans dead/
        let dead = storage.list_dead().unwrap();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].id, id);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_anticheat_corrupted_enc_in_alive_is_removed() {
        // Simuler la triche : mettre un fichier .enc invalide dans alive/
        let dir = std::env::temp_dir().join("monster_test_anticheat_corrupted");
        let _ = fs::remove_dir_all(&dir);
        let storage = LocalStorage::new(&dir).unwrap();

        let fake_id = Uuid::new_v4();
        let cheat_path = dir.join("alive").join(format!("{}.enc", fake_id));
        fs::write(&cheat_path, b"fake-encrypted-data").unwrap();

        // Le fichier corrompu doit être supprimé
        let alive = storage.list_alive().unwrap();
        assert!(alive.is_empty(), "Le fichier corrompu doit être ignoré");
        assert!(
            !cheat_path.exists(),
            "Le fichier corrompu doit être supprimé"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
