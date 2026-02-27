//! Indicateur de connexion au serveur — barre de statut persistante.
//!
//! Affiche un petit bandeau en bas de l'écran indiquant si le serveur
//! de jeu est joignable (vert) ou non (rouge).

use std::sync::{Arc, Mutex};
use std::time::Instant;

use bevy::prelude::*;

use crate::ui::common::colors;

// ═══════════════════════════════════════════════════════════════════
//  Ressource de connexion
// ═══════════════════════════════════════════════════════════════════

/// Adresse du serveur de jeu.
const SERVER_ADDR: &str = "monster-battle.darthoit.eu";

/// Intervalle entre deux vérifications (secondes).
const CHECK_INTERVAL_SECS: f32 = 30.0;

/// État de la connexion au serveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    /// Pas encore vérifié.
    Unknown,
    /// Le serveur est joignable.
    Online,
    /// Le serveur est injoignable.
    Offline,
}

/// Ressource Bevy contenant l'état de la connexion.
#[derive(Resource)]
pub struct ConnectionState {
    pub status: ServerStatus,
    pub last_check: Option<Instant>,
    /// Partagé avec le thread de vérification.
    shared: Arc<Mutex<Option<bool>>>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            status: ServerStatus::Unknown,
            last_check: None,
            shared: Arc::new(Mutex::new(None)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Marqueurs UI
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour le nœud racine de la barre de statut.
#[derive(Component)]
struct StatusBar;

/// Marqueur pour le texte de statut (pour mise à jour).
#[derive(Component)]
struct StatusText;

/// Marqueur pour le point coloré de statut.
#[derive(Component)]
struct StatusDot;

// ═══════════════════════════════════════════════════════════════════
//  Plugin
// ═══════════════════════════════════════════════════════════════════

/// Plugin de statut de connexion.
pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConnectionState::default())
            .add_systems(Startup, spawn_status_bar)
            .add_systems(Update, (trigger_health_check, update_status_bar));
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Systèmes
// ═══════════════════════════════════════════════════════════════════

/// Crée la barre de statut persistante (overlay absolu en bas).
fn spawn_status_bar(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(4.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            // Pas de fond — discret
            GlobalZIndex(100), // Au-dessus de tout le reste
            StatusBar,
        ))
        .with_children(|bar| {
            // Petit rond coloré
            bar.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(colors::TEXT_SECONDARY),
                StatusDot,
            ));

            // Texte de statut
            bar.spawn((
                Text::new("Serveur: ..."),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                StatusText,
            ));
        });
}

/// Lance une vérification de santé du serveur en arrière-plan (thread natif).
fn trigger_health_check(mut conn: ResMut<ConnectionState>) {
    let now = Instant::now();

    // Vérifier si on a besoin d'un nouveau check
    let needs_check = match conn.last_check {
        None => true,
        Some(last) => now.duration_since(last).as_secs_f32() >= CHECK_INTERVAL_SECS,
    };

    // Récupérer le résultat d'un check précédent
    let pending_result = conn
        .shared
        .try_lock()
        .ok()
        .and_then(|mut guard| guard.take());

    if let Some(result) = pending_result {
        conn.status = if result {
            ServerStatus::Online
        } else {
            ServerStatus::Offline
        };
    }

    if needs_check {
        conn.last_check = Some(now);
        let shared = conn.shared.clone();
        let addr = SERVER_ADDR.to_string();

        // Lancer la vérification dans un thread natif (pas de tokio runtime ici)
        std::thread::spawn(move || {
            let result = std::net::TcpStream::connect_timeout(
                &format!("{}:443", addr)
                    .to_socket_addrs_simple()
                    .unwrap_or_else(|| std::net::SocketAddr::from(([0, 0, 0, 0], 443))),
                std::time::Duration::from_secs(5),
            )
            .is_ok();

            if let Ok(mut guard) = shared.lock() {
                *guard = Some(result);
            }
        });
    }
}

/// Met à jour la barre de statut en fonction de l'état de connexion.
fn update_status_bar(
    conn: Res<ConnectionState>,
    mut text_query: Query<&mut Text, With<StatusText>>,
    mut dot_query: Query<&mut BackgroundColor, With<StatusDot>>,
) {
    if !conn.is_changed() {
        return;
    }

    let (label, color) = match conn.status {
        ServerStatus::Unknown => ("Serveur: ...", colors::TEXT_SECONDARY),
        ServerStatus::Online => ("Serveur: connecte", colors::ACCENT_GREEN),
        ServerStatus::Offline => ("Serveur: hors ligne", colors::ACCENT_RED),
    };

    for mut text in &mut text_query {
        *text = Text::new(label);
    }
    for mut bg in &mut dot_query {
        *bg = BackgroundColor(color);
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Helper pour résolution DNS simple
// ═══════════════════════════════════════════════════════════════════

trait ToSocketAddrsSimple {
    fn to_socket_addrs_simple(&self) -> Option<std::net::SocketAddr>;
}

impl ToSocketAddrsSimple for str {
    fn to_socket_addrs_simple(&self) -> Option<std::net::SocketAddr> {
        use std::net::ToSocketAddrs;
        self.to_socket_addrs().ok()?.next()
    }
}
