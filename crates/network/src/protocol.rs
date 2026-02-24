use serde::{Deserialize, Serialize};

use monster_battle_core::Monster;
use monster_battle_core::battle::BattleMessage;

/// Type d'action multijoueur.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetAction {
    /// Combat PvP.
    Combat,
    /// Reproduction.
    Breed,
}

/// Messages échangés entre un client et le serveur relais.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    // ── Client → Serveur ────────────────────────────
    /// S'inscrire dans la file d'attente pour un combat ou une reproduction.
    Queue {
        action: NetAction,
        monster: Monster,
        player_name: String,
    },
    /// Quitter la file d'attente.
    CancelQueue,
    /// Choix d'attaque du joueur (PvP interactif).
    PvpAttackChoice { attack_index: usize },

    // ── Serveur → Client ────────────────────────────
    /// Confirmation de mise en file d'attente.
    Queued,
    /// Un adversaire / partenaire a été trouvé.
    Matched { opponent_name: String },
    /// Résultat d'un combat PvP (calculé par le serveur).
    CombatResult {
        winner_id: String,
        loser_id: String,
        loser_died: bool,
        log: Vec<String>,
        /// Monstre du joueur mis à jour après combat.
        updated_monster: Monster,
    },
    /// Monstre de l'adversaire PvP (envoyé à chaque joueur pour affichage).
    CombatOpponent { opponent_monster: Monster },
    /// Résultat d'un tour PvP (messages + état).
    PvpTurnResult {
        /// Messages de combat à afficher.
        messages: Vec<BattleMessage>,
        /// PV actuels du joueur (de la perspective du destinataire).
        player_hp: u32,
        /// PV actuels de l'adversaire (de la perspective du destinataire).
        opponent_hp: u32,
        /// Le combat est-il terminé ?
        battle_over: bool,
        /// Le destinataire a-t-il gagné ? (pertinent si `battle_over`).
        victory: bool,
        /// XP gagné (pertinent si `battle_over` et `victory`).
        xp_gained: u32,
        /// Le perdant est-il mort ? (pertinent si `battle_over`).
        loser_died: bool,
    },
    /// Données du monstre partenaire (reproduction — envoyé à chaque joueur).
    BreedingPartner { partner_monster: Monster },

    // ── Bidirectionnel ──────────────────────────────
    /// Ping pour vérifier la connexion.
    Ping,
    /// Pong en réponse.
    Pong,
    /// Déconnexion propre.
    Disconnect,
    /// Erreur.
    Error(String),
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
