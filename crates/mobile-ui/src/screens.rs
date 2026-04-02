//! Sous-modules d'écrans additionnels — stubs à implémenter.
//!
//! Chaque écran suit le même pattern :
//! - `spawn_*` : construit l'UI (système `OnEnter`)
//! - `handle_*_input` : gère les entrées (système `Update`)
//!
//! Les écrans utilisent le marqueur [`ScreenEntity`](crate::game::ScreenEntity)
//! pour le nettoyage automatique à la sortie.

// TODO: Implémenter les écrans suivants :
// - new_monster.rs     : choix du type + naming
// - training.rs        : entraînement (docile / sauvage)
// - select_monster.rs  : sélection du monstre (mutualisé)
// - pvp.rs             : recherche PvP + matchmaking
// - breeding.rs        : reproduction + naming du bébé
// - cemetery.rs        : liste des monstres morts
// - help.rs            : tutoriel / aide
