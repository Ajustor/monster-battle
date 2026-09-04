//! Serveur Monster Battle : relais de combat, comptes joueurs et
//! synchronisation multi-appareils des monstres.
//!
//! Le binaire monte les deux sur un seul port ; cette bibliothèque expose les
//! briques pour qu'elles soient testables indépendamment.

pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod monsters;
pub mod relay;
