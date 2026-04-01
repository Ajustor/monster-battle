use bevy::prelude::*;
use std::path::PathBuf;

/// Configuration injectée par chaque adapter (Android, iOS, Desktop).
/// Les écrans lisent cette resource au lieu de constantes compile-time.
#[derive(Resource, Clone)]
pub struct PlatformConfig {
    /// Marge haute (notch, caméra, barre de statut).
    pub safe_top: f32,
    /// Marge basse (home indicator, boutons système).
    pub safe_bottom: f32,
    /// Répertoire de données persistantes.
    pub data_dir: PathBuf,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            safe_top: 16.0,
            safe_bottom: 16.0,
            data_dir: PathBuf::from("."),
        }
    }
}
