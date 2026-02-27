//! Composants UI communs — en-tête, pied de page, styles partagés.

use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::text::Font;

use crate::game::ScreenEntity;

// ═══════════════════════════════════════════════════════════════════
//  Police personnalisée (DejaVu Sans — support Latin complet)
// ═══════════════════════════════════════════════════════════════════

/// Données de la police DejaVu Sans embarquée dans le binaire.
const FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");

/// Système de démarrage : remplace la police par défaut de Bevy
/// par DejaVu Sans (support complet des accents français et symboles).
pub fn setup_custom_font(mut fonts: ResMut<Assets<Font>>) {
    let font = Font::try_from_bytes(FONT_DATA.to_vec())
        .expect("Impossible de charger la police DejaVu Sans");
    // Remplacer la police à l'ID par défaut → tous les TextFont::default() l'utiliseront
    fonts.insert(Handle::<Font>::default().id(), font);
    log::info!("Police DejaVu Sans chargee avec succes");
}

// ═══════════════════════════════════════════════════════════════════
//  Marge de sécurité pour l'encoche caméra (safe area)
// ═══════════════════════════════════════════════════════════════════

/// Marge haute pour éviter l'encoche caméra sur Android.
#[cfg(target_os = "android")]
pub const SAFE_TOP: f32 = 48.0;
#[cfg(not(target_os = "android"))]
pub const SAFE_TOP: f32 = 16.0;

/// Marge basse pour éviter de masquer les boutons derrière la barre de statut.
#[cfg(target_os = "android")]
pub const SAFE_BOTTOM: f32 = 52.0;
#[cfg(not(target_os = "android"))]
pub const SAFE_BOTTOM: f32 = 16.0;

// ═══════════════════════════════════════════════════════════════════
//  Constantes de style
// ═══════════════════════════════════════════════════════════════════

/// Couleurs du jeu.
pub mod colors {
    use bevy::prelude::*;

    pub const BACKGROUND: Color = Color::srgb(0.08, 0.08, 0.12);
    pub const PANEL: Color = Color::srgb(0.12, 0.12, 0.18);
    pub const BORDER: Color = Color::srgb(0.25, 0.25, 0.35);
    pub const TEXT_PRIMARY: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);
    pub const ACCENT_YELLOW: Color = Color::srgb(1.0, 0.84, 0.0);
    pub const ACCENT_RED: Color = Color::srgb(0.96, 0.26, 0.21);
    pub const ACCENT_GREEN: Color = Color::srgb(0.30, 0.69, 0.31);
    pub const ACCENT_BLUE: Color = Color::srgb(0.13, 0.59, 0.95);
    pub const ACCENT_MAGENTA: Color = Color::srgb(0.61, 0.15, 0.69);

    pub const HP_HIGH: Color = Color::srgb(0.30, 0.69, 0.31);
    pub const HP_MID: Color = Color::srgb(1.0, 0.84, 0.0);
    pub const HP_LOW: Color = Color::srgb(0.96, 0.26, 0.21);
}

/// Tailles de police.
pub mod fonts {
    pub const TITLE: f32 = 28.0;
    pub const HEADING: f32 = 22.0;
    pub const BODY: f32 = 18.0;
    pub const SMALL: f32 = 14.0;
}

// ═══════════════════════════════════════════════════════════════════
//  Composants helper
// ═══════════════════════════════════════════════════════════════════

/// Crée un nœud racine plein écran pour un écran.
/// Inclut une marge haute pour éviter l'encoche caméra sur Android.
pub fn screen_root() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        padding: UiRect::new(
            Val::Px(16.0),
            Val::Px(16.0),
            Val::Px(SAFE_TOP),
            Val::Px(SAFE_BOTTOM),
        ),
        ..default()
    }
}

/// Crée un en-tête « Monster Battle ».
pub fn spawn_header(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Text::new("~ Monster Battle ~"),
            TextFont {
                font_size: fonts::TITLE,
                ..default()
            },
            TextColor(colors::ACCENT_YELLOW),
            Node {
                margin: UiRect::bottom(Val::Px(24.0)),
                ..default()
            },
            ScreenEntity,
        ))
        .id()
}

/// Crée un bouton de menu avec texte.
pub fn spawn_menu_button(commands: &mut Commands, text: &str, selected: bool) -> Entity {
    let bg_color = if selected {
        colors::ACCENT_YELLOW
    } else {
        colors::PANEL
    };
    let text_color = if selected {
        Color::BLACK
    } else {
        colors::TEXT_PRIMARY
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(90.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(8.0)),
            ScreenEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text.to_string()),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(text_color),
            ));
        })
        .id()
}

/// Crée un pied de page avec texte d'aide.
pub fn spawn_footer(commands: &mut Commands, text: &str) -> Entity {
    commands
        .spawn((
            Text::new(text.to_string()),
            TextFont {
                font_size: fonts::SMALL,
                ..default()
            },
            TextColor(colors::TEXT_SECONDARY),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                ..default()
            },
            ScreenEntity,
        ))
        .id()
}

/// Retourne la couleur de la barre de PV selon le pourcentage.
pub fn hp_color(current: u32, max: u32) -> Color {
    if max == 0 {
        return colors::HP_LOW;
    }
    let ratio = current as f32 / max as f32;
    if ratio > 0.5 {
        colors::HP_HIGH
    } else if ratio > 0.2 {
        colors::HP_MID
    } else {
        colors::HP_LOW
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Scroll tactile pour Android
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour les conteneurs avec scroll tactile.
#[derive(Component)]
pub struct ScrollableContent;

/// État du scroll tactile (suivi du doigt).
#[derive(Resource, Default)]
pub struct TouchScrollState {
    pub active_touch: Option<u64>,
    pub start_y: f32,
    pub last_y: f32,
    pub is_scrolling: bool,
}

/// Seuil de déplacement avant de déclencher le scroll (évite les taps accidentels).
const SCROLL_DRAG_THRESHOLD: f32 = 8.0;

/// Système de scroll tactile — convertit les glissements en défilement.
pub fn handle_touch_scroll(
    mut touch_events: EventReader<TouchInput>,
    mut state: ResMut<TouchScrollState>,
    mut scroll_query: Query<&mut ScrollPosition, With<ScrollableContent>>,
) {
    for event in touch_events.read() {
        match event.phase {
            TouchPhase::Started => {
                if state.active_touch.is_none() {
                    state.active_touch = Some(event.id);
                    state.start_y = event.position.y;
                    state.last_y = event.position.y;
                    state.is_scrolling = false;
                }
            }
            TouchPhase::Moved => {
                if state.active_touch == Some(event.id) {
                    let total_delta = (event.position.y - state.start_y).abs();
                    if total_delta > SCROLL_DRAG_THRESHOLD {
                        state.is_scrolling = true;
                    }

                    if state.is_scrolling {
                        let delta = event.position.y - state.last_y;
                        for mut scroll_pos in &mut scroll_query {
                            scroll_pos.offset_y = (scroll_pos.offset_y - delta).max(0.0);
                        }
                    }
                    state.last_y = event.position.y;
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if state.active_touch == Some(event.id) {
                    state.active_touch = None;
                    state.is_scrolling = false;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Clavier système Android
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour le champ de saisie texte (tap = ouvrir clavier).
#[derive(Component)]
pub struct TextInputField;

/// Marqueur pour le texte affiché dans le champ de saisie.
/// Mis à jour automatiquement par `update_input_display`.
#[derive(Component)]
pub struct InputDisplayText;

/// Affiche le clavier système Android via JNI (plus fiable que show_soft_input).
pub fn show_system_keyboard() {
    #[cfg(target_os = "android")]
    {
        // Essayer d'abord via l'API native
        if let Some(app) = bevy::window::ANDROID_APP.get() {
            app.show_soft_input(true);
        }
        // Puis via JNI comme méthode plus fiable
        show_keyboard_jni();
        log::info!("⌨️ Clavier système Android affiché");
    }
}

/// Masque le clavier système Android via JNI.
pub fn hide_system_keyboard() {
    #[cfg(target_os = "android")]
    {
        if let Some(app) = bevy::window::ANDROID_APP.get() {
            app.hide_soft_input(false);
        }
        hide_keyboard_jni();
        log::info!("⌨️ Clavier système Android masqué");
    }
}

/// Affiche le clavier via JNI : InputMethodManager.showSoftInput(decorView, SHOW_IMPLICIT).
/// Plus fiable que ANativeActivity_showSoftInput sur de nombreux appareils.
#[cfg(target_os = "android")]
fn show_keyboard_jni() {
    let Some(app) = bevy::window::ANDROID_APP.get() else {
        return;
    };

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    // Safety: vm_as_ptr() renvoie le pointeur JVM valide géré par android-activity.
    // On ne doit PAS laisser le wrapper JavaVM être droppé (il appellerait DestroyJavaVM).
    let vm = match unsafe { jni::JavaVM::from_raw(vm_ptr) } {
        Ok(vm) => vm,
        Err(e) => {
            log::error!("JNI show_keyboard: impossible d'obtenir JavaVM: {e}");
            return;
        }
    };

    let result = (|| -> Result<(), jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        // Safety: activity_as_ptr() renvoie une référence globale JNI — ne pas la supprimer.
        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { jni::objects::JObject::from_raw(activity_ptr) };

        // Obtenir la fenêtre : activity.getWindow()
        let window = env
            .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?
            .l()?;

        // Obtenir la vue décorative : window.getDecorView()
        let decor_view = env
            .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
            .l()?;

        // Donner le focus à la vue (requis pour showSoftInput)
        env.call_method(&decor_view, "requestFocus", "()Z", &[])?;

        // Obtenir InputMethodManager via getSystemService("input_method")
        let service_name = env.new_string("input_method")?;
        let imm = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[jni::objects::JValue::Object(&service_name)],
            )?
            .l()?;

        // Appeler showSoftInput(decorView, SHOW_IMPLICIT=1)
        env.call_method(
            &imm,
            "showSoftInput",
            "(Landroid/view/View;I)Z",
            &[
                jni::objects::JValue::Object(&decor_view),
                jni::objects::JValue::Int(1), // SHOW_IMPLICIT
            ],
        )?;

        log::info!("JNI: showSoftInput appelé avec succès");
        Ok(())
    })();

    if let Err(e) = result {
        log::error!("JNI show_keyboard: erreur: {e}");
    }

    // Empêcher le wrapper JavaVM d'appeler DestroyJavaVM
    std::mem::forget(vm);
}

/// Masque le clavier via JNI : InputMethodManager.hideSoftInputFromWindow().
#[cfg(target_os = "android")]
fn hide_keyboard_jni() {
    let Some(app) = bevy::window::ANDROID_APP.get() else {
        return;
    };

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = match unsafe { jni::JavaVM::from_raw(vm_ptr) } {
        Ok(vm) => vm,
        Err(e) => {
            log::error!("JNI hide_keyboard: impossible d'obtenir JavaVM: {e}");
            return;
        }
    };

    let result = (|| -> Result<(), jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { jni::objects::JObject::from_raw(activity_ptr) };

        let window = env
            .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?
            .l()?;

        let decor_view = env
            .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
            .l()?;

        // Obtenir le window token
        let window_token = env
            .call_method(&decor_view, "getWindowToken", "()Landroid/os/IBinder;", &[])?
            .l()?;

        let service_name = env.new_string("input_method")?;
        let imm = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[jni::objects::JValue::Object(&service_name)],
            )?
            .l()?;

        // hideSoftInputFromWindow(windowToken, 0)
        env.call_method(
            &imm,
            "hideSoftInputFromWindow",
            "(Landroid/os/IBinder;I)Z",
            &[
                jni::objects::JValue::Object(&window_token),
                jni::objects::JValue::Int(0),
            ],
        )?;

        Ok(())
    })();

    if let Err(e) = result {
        log::error!("JNI hide_keyboard: erreur: {e}");
    }

    std::mem::forget(vm);
}

// ═══════════════════════════════════════════════════════════════════
//  Timer de ré-essai clavier (le clavier peut échouer si la vue
//  n'est pas encore prête au moment de OnEnter)
// ═══════════════════════════════════════════════════════════════════

/// Ressource : compte les tentatives de ré-ouverture du clavier.
#[derive(Resource)]
pub struct KeyboardRetryTimer {
    pub frames_remaining: u32,
}

/// Système : ré-essaye d'ouvrir le clavier pendant quelques frames.
pub fn retry_show_keyboard(mut timer: ResMut<KeyboardRetryTimer>) {
    if timer.frames_remaining > 0 {
        timer.frames_remaining -= 1;
        show_system_keyboard();
    }
}

/// Système : ouvre le clavier système quand on tape sur un TextInputField.
pub fn handle_input_field_tap(
    query: Query<&Interaction, (Changed<Interaction>, With<TextInputField>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show_system_keyboard();
        }
    }
}

/// Système : synchronise le texte affiché dans le champ de saisie
/// avec la valeur courante de `GameData::name_input`.
pub fn update_input_display(
    data: Res<crate::game::GameData>,
    mut query: Query<(&mut Text, &mut TextColor), With<InputDisplayText>>,
) {
    if !data.is_changed() {
        return;
    }
    for (mut text, mut color) in &mut query {
        if data.name_input.is_empty() {
            *text = Text::new("Toucher pour saisir...");
            *color = TextColor(colors::TEXT_SECONDARY);
        } else {
            *text = Text::new(format!("{}|", data.name_input));
            *color = TextColor(colors::TEXT_PRIMARY);
        }
    }
}
