//! Mise à jour automatique de l'application.
//!
//! Utilise le `DownloadManager` Android pour télécharger le nouvel APK
//! en arrière-plan. Une notification système affiche la progression,
//! et l'utilisateur peut installer la mise à jour en appuyant dessus.

/// URL de téléchargement de l'APK.
const APK_URL: &str = "https://ajustor.github.io/monster-battle/monster-battle-android.apk";

/// Lance le téléchargement de la mise à jour via le DownloadManager Android.
///
/// Le téléchargement s'effectue en arrière-plan par le système.
/// Une notification apparaît pour suivre la progression.
/// Une fois terminé, l'utilisateur peut installer la mise à jour
/// en appuyant sur la notification.
///
/// Retourne `true` si le téléchargement a été lancé avec succès.
pub fn download_update(server_version: &str) -> bool {
    #[cfg(target_os = "android")]
    {
        match android_download_update(server_version) {
            Ok(id) => {
                log::info!("📥 Téléchargement lancé (download_id={})", id);
                true
            }
            Err(e) => {
                log::error!("❌ Impossible de lancer le téléchargement : {}", e);
                false
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = server_version;
        log::info!("📦 Mise à jour : ouverture dans le navigateur (desktop)");
        let _ = std::process::Command::new("xdg-open").arg(APK_URL).spawn();
        true
    }
}

/// Implémentation Android via JNI → DownloadManager.
#[cfg(target_os = "android")]
fn android_download_update(server_version: &str) -> Result<i64, String> {
    use jni::objects::{JObject, JValue};

    let app = bevy::window::ANDROID_APP
        .get()
        .ok_or("Impossible d'obtenir l'AndroidApp")?;

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm =
        unsafe { jni::JavaVM::from_raw(vm_ptr) }.map_err(|e| format!("JavaVM::from_raw: {}", e))?;

    let result = (|| -> Result<i64, jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { JObject::from_raw(activity_ptr) };

        // ── Obtenir le DownloadManager ──────────────────────────
        let service_name = env.new_string("download")?;
        let dm = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;

        // ── Créer l'URI de téléchargement ───────────────────────
        let url = env.new_string(APK_URL)?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&url)],
            )?
            .l()?;

        // ── Créer la requête DownloadManager.Request ────────────
        let request = env.new_object(
            "android/app/DownloadManager$Request",
            "(Landroid/net/Uri;)V",
            &[JValue::Object(&uri)],
        )?;

        // Titre de la notification
        let title = env.new_string("Monster Battle — Mise a jour")?;
        env.call_method(
            &request,
            "setTitle",
            "(Ljava/lang/CharSequence;)Landroid/app/DownloadManager$Request;",
            &[JValue::Object(&title)],
        )?;

        // Description
        let desc = env.new_string(format!("Version {}", server_version))?;
        env.call_method(
            &request,
            "setDescription",
            "(Ljava/lang/CharSequence;)Landroid/app/DownloadManager$Request;",
            &[JValue::Object(&desc)],
        )?;

        // Notification visible pendant et après le téléchargement
        // VISIBILITY_VISIBLE_NOTIFY_COMPLETED = 1
        env.call_method(
            &request,
            "setNotificationVisibility",
            "(I)Landroid/app/DownloadManager$Request;",
            &[JValue::Int(1)],
        )?;

        // Type MIME pour que Android propose l'installation
        let mime = env.new_string("application/vnd.android.package-archive")?;
        env.call_method(
            &request,
            "setMimeType",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&mime)],
        )?;

        // Destination : dossier Downloads (accessible sans permission spéciale
        // via le DownloadManager qui dispose de privilèges système)
        let downloads_dir = env
            .get_static_field(
                "android/os/Environment",
                "DIRECTORY_DOWNLOADS",
                "Ljava/lang/String;",
            )?
            .l()?;
        let filename = env.new_string("monster-battle.apk")?;
        env.call_method(
            &request,
            "setDestinationInExternalPublicDir",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/app/DownloadManager$Request;",
            &[JValue::Object(&downloads_dir), JValue::Object(&filename)],
        )?;

        // ── Lancer le téléchargement ────────────────────────────
        let download_id = env
            .call_method(
                &dm,
                "enqueue",
                "(Landroid/app/DownloadManager$Request;)J",
                &[JValue::Object(&request)],
            )?
            .j()?;

        // Ne pas libérer activity — c'est un ptr emprunté
        std::mem::forget(activity);

        Ok(download_id)
    })();

    // Empêcher le wrapper JavaVM d'appeler DestroyJavaVM
    std::mem::forget(vm);

    result.map_err(|e| format!("JNI error: {}", e))
}
