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
//  Résolution DNS via JNI (contourne le getaddrinfo Android défaillant)
// ═══════════════════════════════════════════════════════════════════

/// Résout un nom d'hôte en adresse IP via JNI (`InetAddress.getByName`).
///
/// Sur Android, `getaddrinfo` depuis du code natif peut échouer alors que
/// la résolution DNS Java fonctionne normalement. Cette fonction appelle
/// le résolveur DNS de la JVM pour contourner le problème.
#[cfg(target_os = "android")]
pub fn resolve_host_jni(hostname: &str) -> Option<std::net::IpAddr> {
    use jni::objects::JValue;

    let app = bevy::window::ANDROID_APP.get()?;
    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }.ok()?;

    let result = (|| -> Result<std::net::IpAddr, Box<dyn std::error::Error>> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let hostname_jstr = env.new_string(hostname)?;

        // InetAddress addr = InetAddress.getByName(hostname);
        let inet_address = env
            .call_static_method(
                "java/net/InetAddress",
                "getByName",
                "(Ljava/lang/String;)Ljava/net/InetAddress;",
                &[JValue::Object(&hostname_jstr.into())],
            )?
            .l()?;

        // String ip = addr.getHostAddress();
        let ip_jstr = env
            .call_method(&inet_address, "getHostAddress", "()Ljava/lang/String;", &[])?
            .l()?;

        let ip_string: String = env.get_string((&ip_jstr).into())?.into();
        let ip: std::net::IpAddr = ip_string.parse()?;

        Ok(ip)
    })();

    // Ne pas appeler DestroyJavaVM — la JVM appartient à Android
    std::mem::forget(vm);

    match result {
        Ok(ip) => {
            log::info!("🔍 DNS résolu via JNI : {} → {}", hostname, ip);
            Some(ip)
        }
        Err(e) => {
            log::error!("❌ Résolution DNS JNI échouée pour {} : {}", hostname, e);
            None
        }
    }
}

/// Fallback pour les builds non-Android (utilise la résolution système standard).
#[cfg(not(target_os = "android"))]
pub fn resolve_host_jni(hostname: &str) -> Option<std::net::IpAddr> {
    use std::net::ToSocketAddrs;
    let addr = format!("{}:443", hostname);
    addr.to_socket_addrs().ok()?.next().map(|sa| sa.ip())
}

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

/// Marqueur pour le nœud racine de la modale de mise à jour.
#[derive(Component)]
struct UpdateModal;

/// Marqueur pour le bouton "Fermer" de la modale.
#[derive(Component)]
struct UpdateModalDismiss;

/// Ressource de suivi : la modale a déjà été affichée.
#[derive(Resource, Default)]
struct UpdateModalShown(bool);

/// État du téléchargement de la mise à jour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadStatus {
    /// Pas encore lancé.
    Idle,
    /// Téléchargement en cours (notification système).
    Downloading,
    /// Le lancement du téléchargement a échoué.
    Failed,
    /// Téléchargement terminé, APK prêt à installer.
    Ready,
}

/// Ressource de suivi du téléchargement.
#[derive(Resource)]
struct UpdateDownloadState {
    status: DownloadStatus,
    /// Identifiant retourné par DownloadManager.enqueue() — utilisé pour le polling.
    download_id: Option<i64>,
}

impl Default for UpdateDownloadState {
    fn default() -> Self {
        Self {
            status: DownloadStatus::Idle,
            download_id: None,
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
            .insert_resource(VersionState::default())
            .insert_resource(UpdateModalShown::default())
            .insert_resource(UpdateDownloadState::default())
            .add_systems(Startup, spawn_status_bar)
            .add_systems(
                Update,
                (
                    trigger_health_check,
                    update_status_bar,
                    trigger_version_check,
                    handle_update_button,
                    show_update_modal,
                    handle_modal_dismiss,
                    poll_download_complete,
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
                bottom: Val::Px(32.0),
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

        // Lancer la vérification dans un thread natif avec runtime tokio.
        // Résolution DNS via JNI pour contourner le getaddrinfo Android.
        std::thread::spawn(move || {
            // Résoudre le DNS avant de lancer le runtime tokio
            let resolved_ip = resolve_host_jni(&addr);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            let result = match (rt, resolved_ip) {
                (Ok(rt), Some(ip)) => rt.block_on(async {
                    tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        monster_battle_network::check_server_health_resolved(&addr, ip),
                    )
                    .await
                    .unwrap_or(false)
                }),
                _ => false,
            };

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

    let client_version = env!("CARGO_PKG_VERSION");

    let (label, color) = match conn.status {
        ServerStatus::Unknown => (
            format!("v{} | Serveur: ...", client_version),
            colors::TEXT_SECONDARY,
        ),
        ServerStatus::Online => (
            format!("v{} | Serveur: connecte", client_version),
            colors::ACCENT_GREEN,
        ),
        ServerStatus::Offline => (
            format!("v{} | Serveur: hors ligne", client_version),
            colors::ACCENT_RED,
        ),
    };

    for mut text in &mut text_query {
        *text = Text::new(label.clone());
    }
    for mut bg in &mut dot_query {
        *bg = BackgroundColor(color);
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
            // Résoudre le DNS avant de lancer le runtime tokio
            let resolved_ip = resolve_host_jni(&addr);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            let result = match (rt, resolved_ip) {
                (Ok(rt), Some(ip)) => rt.block_on(async {
                    tokio::time::timeout(
                        std::time::Duration::from_secs(10),
                        monster_battle_network::check_server_version_resolved(&addr, ip),
                    )
                    .await
                    .unwrap_or(None)
                }),
                _ => None,
            };

            if let Ok(mut guard) = shared.lock() {
                *guard = Some(result);
            }
        });
    }
}

/// Gère le toucher sur le bouton de mise à jour — lance le téléchargement.
fn handle_update_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<UpdateButton>)>,
    version: Res<VersionState>,
    mut download_state: ResMut<UpdateDownloadState>,
    modal_query: Query<Entity, With<UpdateModal>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Extraire la version serveur pour l'afficher dans la notification
            let sv = match &version.status {
                VersionStatus::UpdateRequired { server_version } => server_version.clone(),
                _ => "?".to_string(),
            };

            let result = crate::updater::start_download(&sv);
            match result {
                Ok(id) => {
                    log::info!("📥 Téléchargement lancé (id={})", id);
                    download_state.status = DownloadStatus::Downloading;
                    download_state.download_id = Some(id);
                    // Fermer la modale
                    for entity in &modal_query {
                        commands.entity(entity).despawn_recursive();
                    }
                }
                Err(e) => {
                    log::error!("❌ Échec lancement téléchargement : {}", e);
                    download_state.status = DownloadStatus::Failed;
                    download_state.download_id = None;
                }
            }
        }
    }
}

/// Ouvre l'URL de mise à jour dans le navigateur.
#[allow(dead_code)]
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

// ═══════════════════════════════════════════════════════════════════
//  Modale de mise à jour — systèmes
// ═══════════════════════════════════════════════════════════════════

/// Surveille le changement de `VersionState` → `UpdateRequired` et affiche
/// une modale par-dessus tout le reste.
fn show_update_modal(
    mut commands: Commands,
    version: Res<VersionState>,
    mut shown: ResMut<UpdateModalShown>,
    existing: Query<Entity, With<UpdateModal>>,
) {
    // Ne rien faire si pas de changement ou déjà affichée
    if !version.is_changed() || shown.0 {
        return;
    }

    let VersionStatus::UpdateRequired { ref server_version } = version.status else {
        return;
    };

    // Nettoyer d'éventuels doublons
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }

    shown.0 = true;
    let client_version = env!("CARGO_PKG_VERSION");
    let sv = server_version.clone();

    log::info!(
        "📢 Affichage de la modale de mise à jour (client={}, serveur={})",
        client_version,
        sv
    );

    // Fond semi-transparent couvrant tout l'écran
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(200), // Au-dessus de la barre de statut
            UpdateModal,
        ))
        .with_children(|overlay| {
            // Carte de la modale
            overlay
                .spawn((
                    Node {
                        width: Val::Percent(85.0),
                        max_width: Val::Px(400.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|card| {
                    // Titre
                    card.spawn((
                        Text::new("Mise a jour requise !"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(colors::ACCENT_YELLOW),
                    ));

                    // Versions
                    card.spawn((
                        Text::new(format!("Votre version : {}", client_version)),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(colors::ACCENT_RED),
                    ));
                    card.spawn((
                        Text::new(format!("Version serveur : {}", sv)),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(colors::ACCENT_GREEN),
                    ));

                    // Séparateur
                    card.spawn((
                        Node {
                            width: Val::Percent(80.0),
                            height: Val::Px(1.0),
                            margin: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(colors::BORDER),
                    ));

                    // Explication
                    card.spawn((
                        Text::new("Appuyez sur Installer pour telecharger et installer la mise a jour automatiquement."),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(colors::TEXT_SECONDARY),
                        Node {
                            max_width: Val::Px(320.0),
                            ..default()
                        },
                    ));

                    // Bouton « Installer »
                    card.spawn((
                        Node {
                            width: Val::Percent(80.0),
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(colors::ACCENT_YELLOW),
                        BorderRadius::all(Val::Px(8.0)),
                        UpdateButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Installer la mise a jour"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                        ));
                    });

                    // Bouton « Fermer »
                    card.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.3, 0.3, 0.4, 0.6)),
                        BorderRadius::all(Val::Px(6.0)),
                        UpdateModalDismiss,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Fermer"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(colors::TEXT_SECONDARY),
                        ));
                    });
                });
        });
}

/// Détruit la modale lorsque le bouton « Fermer » est pressé.
fn handle_modal_dismiss(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<UpdateModalDismiss>)>,
    modal_query: Query<Entity, With<UpdateModal>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            for entity in &modal_query {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Poll le statut du téléchargement en cours et déclenche l'installation si terminé.
fn poll_download_complete(mut download_state: ResMut<UpdateDownloadState>) {
    let DownloadStatus::Downloading = download_state.status else {
        return;
    };
    let Some(id) = download_state.download_id else {
        return;
    };

    match crate::updater::check_download_complete(id) {
        crate::updater::DownloadPollResult::Complete(path) => {
            log::info!("✅ Téléchargement terminé : {}", path);
            download_state.status = DownloadStatus::Ready;
            download_state.download_id = None;
            crate::updater::trigger_install(&path);
        }
        crate::updater::DownloadPollResult::Failed(reason) => {
            log::error!("❌ Téléchargement échoué : {}", reason);
            download_state.status = DownloadStatus::Failed;
            download_state.download_id = None;
        }
        crate::updater::DownloadPollResult::InProgress => {
            // Toujours en cours, rien à faire
        }
    }
}
