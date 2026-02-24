use serde::{Deserialize, Serialize};

use monster_battle_core::Monster;

/// Messages échangés entre les joueurs en réseau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    /// Proposition d'action (combat ou reproduction).
    Propose(NetAction),
    /// Acceptation d'une proposition.
    Accept,
    /// Refus d'une proposition.
    Decline,
    /// Envoi des données du monstre.
    MonsterData(Monster),
    /// Résultat d'un combat (envoyé par l'hôte).
    CombatResult {
        winner_id: String,
        loser_id: String,
        loser_died: bool,
        log: Vec<String>,
        /// Monstre mis à jour du joueur (après combat).
        updated_monster: Monster,
    },
    /// Résultat d'une reproduction.
    BreedingResult {
        child: Monster,
        description: String,
        /// `true` si l'enfant est pour le destinataire.
        child_is_yours: bool,
    },
    /// Ping pour vérifier la connexion.
    Ping,
    /// Pong en réponse.
    Pong,
    /// Déconnexion propre.
    Disconnect,
    /// Erreur.
    Error(String),
}

/// Type d'action multijoueur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetAction {
    /// Proposition de combat PvP.
    Combat,
    /// Proposition de reproduction.
    Breed,
}

impl NetMessage {
    /// Sérialise le message en bytes (longueur préfixée).
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        let len = json.len() as u32;
        let mut bytes = Vec::with_capacity(4 + json.len());
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(json.as_bytes());
        Ok(bytes)
    }

    /// Désérialise un message depuis un buffer JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
