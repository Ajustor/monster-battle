//! Indicateur de connexion au serveur — barre de statut persistante.
//!
//! Affiche un petit bandeau en bas de l'écran indiquant si le serveur
//! de jeu est joignable (vert) ou non (rouge).
//! Vérifie également la version du serveur au démarrage.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use bevy::prelude::*;

use crate::ui::common::colors;

// ═══════════════════════════════════════════════════════════════════
//  Ressource de connexion
// ═══════════════════════════════════════════════════════════════════

/// Adresse du serveur de jeu.
const SERVER_ADDR: &str = "monster-battle.darthoit.eu";

/// URL de téléchargement des mises à jour.
pub const UPDATE_URL: &str = "https://ajustor.github.io/monster-battle";

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
//  Vérification de version
// ═══════════════════════════════════════════════════════════════════

/// Résultat possible de la vérification de version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionStatus {
    /// Pas encore vérifié.
    Checking,
    /// La version correspond.
    UpToDate,
    /// Mise à jour requise.
    UpdateRequired { server_version: String },
    /// Impossible de vérifier (serveur injoignable).
    CheckFailed,
}

/// Ressource Bevy contenant l'état de la vérification de version.
#[derive(Resource)]
pub struct VersionState {
    pub status: VersionStatus,
    /// Partagé avec le thread de vérification.
    shared: Arc<Mutex<Option<Option<String>>>>,
    /// Vérifié au moins une fois.
    check_started: bool,
}

impl Default for VersionState {
    fn default() -> Self {
        Self {
            status: VersionStatus::Checking,
            shared: Arc::new(Mutex::new(None)),
            check_started: false,
        }
    }
}

/// Marqueur pour la bannière de mise à jour.
#[derive(Component)]
pub struct UpdateBanner;

/// Marqueur pour le bouton de mise à jour.
#[derive(Component)]
pub struct UpdateButton;

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
            .insert_resource(VersionState::default())
            .add_systems(Startup, spawn_status_bar)
            .add_systems(
                Update,
                (
                    trigger_health_check,
                    update_status_bar,
                    trigger_version_check,
                    handle_update_button,
                ),
            );
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
                bottom: Val::Px(8.0),
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

// ═══════════════════════════════════════════════════════════════════
//  Vérification de version — système
// ═══════════════════════════════════════════════════════════════════

/// Lance la vérification de version au démarrage (une seule fois).
fn trigger_version_check(mut version: ResMut<VersionState>) {
    // Récupérer le résultat si le thread a fini
    let pending = version
        .shared
        .try_lock()
        .ok()
        .and_then(|mut guard| guard.take());

    if let Some(result) = pending {
        let client_version = env!("CARGO_PKG_VERSION");
        match result {
            Some(sv) if sv != client_version => {
                log::info!(
                    "🔄 Version mismatch — client={}, serveur={}",
                    client_version,
                    sv
                );
                version.status = VersionStatus::UpdateRequired { server_version: sv };
            }
            Some(_) => {
                log::info!("✅ Version OK — {}", client_version);
                version.status = VersionStatus::UpToDate;
            }
            None => {
                log::warn!("⚠️ Impossible de vérifier la version du serveur");
                version.status = VersionStatus::CheckFailed;
            }
        }
        return;
    }

    // Lancer le check une seule fois
    if !version.check_started {
        version.check_started = true;
        let shared = version.shared.clone();
        let addr = SERVER_ADDR.to_string();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            let result = match rt {
                Ok(rt) => rt.block_on(async {
                    tokio::time::timeout(
                        std::time::Duration::from_secs(10),
                        monster_battle_network::check_server_version(&addr),
                    )
                    .await
                    .unwrap_or(None)
                }),
                Err(_) => None,
            };

            if let Ok(mut guard) = shared.lock() {
                *guard = Some(result);
            }
        });
    }
}

/// Gère le toucher sur le bouton de mise à jour — ouvre le navigateur.
fn handle_update_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<UpdateButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            open_update_url();
        }
    }
}

/// Ouvre l'URL de mise à jour dans le navigateur.
fn open_update_url() {
    #[cfg(target_os = "android")]
    {
        // Sur Android, utiliser JNI pour ouvrir un Intent ACTION_VIEW
        android_open_url(UPDATE_URL);
    }
    #[cfg(not(target_os = "android"))]
    {
        // Sur desktop (dev), utiliser xdg-open
        let _ = std::process::Command::new("xdg-open")
            .arg(UPDATE_URL)
            .spawn();
    }
}

/// Ouvre une URL sur Android via JNI (Intent ACTION_VIEW).
#[cfg(target_os = "android")]
fn android_open_url(url: &str) {
    use jni::objects::{JObject, JValue};

    let Some(app) = bevy::window::ANDROID_APP.get() else {
        log::error!("❌ Impossible d'obtenir l'AndroidApp pour ouvrir l'URL");
        return;
    };

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = match unsafe { jni::JavaVM::from_raw(vm_ptr) } {
        Ok(vm) => vm,
        Err(e) => {
            log::error!("❌ JavaVM::from_raw failed: {}", e);
            return;
        }
    };

    let result = (|| -> Result<(), jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let url_string = env.new_string(url)?;

        // Uri.parse(url)
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&url_string)],
            )?
            .l()?;

        // new Intent(Intent.ACTION_VIEW, uri)
        let action_view = env.new_string("android.intent.action.VIEW")?;
        let intent = env.new_object(
            "android/content/Intent",
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            &[JValue::Object(&action_view), JValue::Object(&uri)],
        )?;

        // intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        env.call_method(
            &intent,
            "addFlags",
            "(I)Landroid/content/Intent;",
            &[JValue::Int(0x10000000)], // FLAG_ACTIVITY_NEW_TASK
        )?;

        // Obtenir l'Activity
        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { JObject::from_raw(activity_ptr) };

        // activity.startActivity(intent)
        env.call_method(
            &activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )?;

        // Ne pas libérer activity — c'est un ptr emprunté
        std::mem::forget(activity);

        log::info!("🌐 URL ouverte: {}", url);
        Ok(())
    })();

    if let Err(e) = result {
        log::error!("❌ android_open_url: erreur JNI: {}", e);
    }

    // Empêcher le wrapper JavaVM d'appeler DestroyJavaVM
    std::mem::forget(vm);
}
