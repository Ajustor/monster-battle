//! Mise à jour de l'application.
//!
//! Ouvre l'URL de téléchargement de l'APK dans le navigateur Android.
//! Approche fiable sur tous les appareils sans nécessiter de permissions spéciales.

/// URL de téléchargement de l'APK.
const APK_URL: &str = "https://ajustor.github.io/monster-battle/monster-battle-android.apk";

/// Résultat du poll (conservé pour compatibilité avec connection.rs).
pub enum DownloadPollResult {
    InProgress,
    Complete(String),
    Failed(String),
}

/// Ouvre l'URL de l'APK dans le navigateur pour lancer le téléchargement.
pub fn start_download(server_version: &str) -> Result<i64, String> {
    log::info!("📦 Ouverture URL mise à jour (version cible: {})", server_version);

    #[cfg(target_os = "android")]
    {
        android_open_url(APK_URL)?;
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = server_version;
        log::info!("📦 Desktop : xdg-open {}", APK_URL);
        let _ = std::process::Command::new("xdg-open").arg(APK_URL).spawn();
    }

    Ok(0) // ID fictif — on ne suit pas le téléchargement
}

/// Pas de polling nécessaire — le téléchargement est géré par le navigateur.
pub fn check_download_complete(_download_id: i64) -> DownloadPollResult {
    DownloadPollResult::InProgress
}

/// Pas d'installation automatique — l'utilisateur installe depuis le navigateur.
pub fn trigger_install(_apk_path: &str) {}

// ════════════════════════════════════════════════════════════════════
//  Android : ouvrir une URL via Intent ACTION_VIEW
// ════════════════════════════════════════════════════════════════════

#[cfg(target_os = "android")]
fn android_open_url(url: &str) -> Result<(), String> {
    use jni::objects::{JObject, JValue};

    let app = bevy::window::ANDROID_APP
        .get()
        .ok_or("AndroidApp indisponible")?;

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }
        .map_err(|e| format!("JavaVM::from_raw: {}", e))?;

    let result = (|| -> Result<(), jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { JObject::from_raw(activity_ptr) };

        // Uri.parse(url)
        let url_str = env.new_string(url)?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&url_str)],
            )?
            .l()?;

        // Intent ACTION_VIEW
        let action = env.new_string("android.intent.action.VIEW")?;
        let intent = env.new_object(
            "android/content/Intent",
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            &[JValue::Object(&action), JValue::Object(&uri)],
        )?;

        // FLAG_ACTIVITY_NEW_TASK = 0x10000000
        env.call_method(
            &intent,
            "addFlags",
            "(I)Landroid/content/Intent;",
            &[JValue::Int(0x10000000i32)],
        )?;

        // startActivity
        env.call_method(
            &activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )?;

        std::mem::forget(activity);
        Ok(())
    })();

    std::mem::forget(vm);
    result.map_err(|e| format!("JNI error: {}", e))
}
