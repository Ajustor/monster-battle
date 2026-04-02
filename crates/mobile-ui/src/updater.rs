//! Mise à jour automatique de l'application.
//!
//! Utilise le `DownloadManager` Android pour télécharger le nouvel APK
//! en arrière-plan. Une notification système affiche la progression.
//! Quand le téléchargement est terminé, l'installation est proposée
//! automatiquement via un Intent.

/// URL de téléchargement de l'APK.
const APK_URL: &str = "https://ajustor.github.io/monster-battle/monster-battle-android.apk";

/// Résultat du poll de l'état d'un téléchargement.
pub enum DownloadPollResult {
    InProgress,
    Complete(String), // chemin local de l'APK
    Failed(String),
}

/// Lance le téléchargement de la mise à jour.
/// Retourne l'identifiant du téléchargement (pour polling) ou une erreur.
pub fn start_download(server_version: &str) -> Result<i64, String> {
    #[cfg(target_os = "android")]
    {
        android_start_download(server_version)
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = server_version;
        log::info!("📦 Mise à jour : ouverture dans le navigateur (desktop)");
        let _ = std::process::Command::new("xdg-open").arg(APK_URL).spawn();
        Ok(0) // ID fictif sur desktop
    }
}

/// Vérifie si un téléchargement est terminé via DownloadManager.
pub fn check_download_complete(download_id: i64) -> DownloadPollResult {
    #[cfg(target_os = "android")]
    {
        android_check_download(download_id)
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = download_id;
        DownloadPollResult::InProgress
    }
}

/// Déclenche l'installation de l'APK via un Intent Android.
pub fn trigger_install(apk_path: &str) {
    #[cfg(target_os = "android")]
    {
        if let Err(e) = android_trigger_install(apk_path) {
            log::error!("❌ Impossible de lancer l'installation : {}", e);
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = apk_path;
        log::info!("📦 Installation non supportée sur desktop");
    }
}

// ════════════════════════════════════════════════════════════════════
//  Implémentations Android JNI
// ════════════════════════════════════════════════════════════════════

#[cfg(target_os = "android")]
fn android_start_download(server_version: &str) -> Result<i64, String> {
    use jni::objects::{JObject, JValue};

    let app = bevy::window::ANDROID_APP
        .get()
        .ok_or("Impossible d'obtenir l'AndroidApp")?;

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }
        .map_err(|e| format!("JavaVM::from_raw: {}", e))?;

    let result = (|| -> Result<i64, jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { JObject::from_raw(activity_ptr) };

        // ── DownloadManager ─────────────────────────────────────
        let service_name = env.new_string("download")?;
        let dm = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;

        // ── URI ─────────────────────────────────────────────────
        let url = env.new_string(APK_URL)?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&url)],
            )?
            .l()?;

        // ── Request ─────────────────────────────────────────────
        let request = env.new_object(
            "android/app/DownloadManager$Request",
            "(Landroid/net/Uri;)V",
            &[JValue::Object(&uri)],
        )?;

        let title = env.new_string("Monster Battle - Mise a jour")?;
        env.call_method(
            &request,
            "setTitle",
            "(Ljava/lang/CharSequence;)Landroid/app/DownloadManager$Request;",
            &[JValue::Object(&title)],
        )?;

        let desc = env.new_string(format!("Version {}", server_version))?;
        env.call_method(
            &request,
            "setDescription",
            "(Ljava/lang/CharSequence;)Landroid/app/DownloadManager$Request;",
            &[JValue::Object(&desc)],
        )?;

        // VISIBILITY_VISIBLE_NOTIFY_COMPLETED = 1
        env.call_method(
            &request,
            "setNotificationVisibility",
            "(I)Landroid/app/DownloadManager$Request;",
            &[JValue::Int(1)],
        )?;

        let mime = env.new_string("application/vnd.android.package-archive")?;
        env.call_method(
            &request,
            "setMimeType",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&mime)],
        )?;

        // Destination dans le cache interne de l'app (pas de permission requise)
        // getFilesDir() retourne /data/data/<package>/files/
        let files_dir = env
            .call_method(&activity, "getFilesDir", "()Ljava/io/File;", &[])?
            .l()?;
        let files_path = env
            .call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        let files_path_str: String = env.get_string(&files_path.into())?.into();
        let dest_path = format!("{}/monster-battle.apk", files_path_str);
        let dest_uri_str = env.new_string(format!("file://{}", dest_path))?;
        let dest_uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&dest_uri_str)],
            )?
            .l()?;
        env.call_method(
            &request,
            "setDestinationUri",
            "(Landroid/net/Uri;)Landroid/app/DownloadManager$Request;",
            &[JValue::Object(&dest_uri)],
        )?;

        let download_id = env
            .call_method(
                &dm,
                "enqueue",
                "(Landroid/app/DownloadManager$Request;)J",
                &[JValue::Object(&request)],
            )?
            .j()?;

        std::mem::forget(activity);
        Ok(download_id)
    })();

    std::mem::forget(vm);
    result.map_err(|e| format!("JNI error: {}", e))
}

#[cfg(target_os = "android")]
fn android_check_download(download_id: i64) -> DownloadPollResult {
    use jni::objects::{JObject, JValue};

    let app = match bevy::window::ANDROID_APP.get() {
        Some(a) => a,
        None => return DownloadPollResult::Failed("AndroidApp indisponible".to_string()),
    };

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = match unsafe { jni::JavaVM::from_raw(vm_ptr) } {
        Ok(v) => v,
        Err(e) => return DownloadPollResult::Failed(e.to_string()),
    };

    let result = (|| -> Result<DownloadPollResult, jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { JObject::from_raw(activity_ptr) };

        let service_name = env.new_string("download")?;
        let dm = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;

        // Créer la query pour cet ID
        let query = env.new_object(
            "android/app/DownloadManager$Query",
            "()V",
            &[],
        )?;

        let ids_arr = env.new_long_array(1)?;
        env.set_long_array_region(&ids_arr, 0, &[download_id])?;
        env.call_method(
            &query,
            "setFilterById",
            "([J)Landroid/app/DownloadManager$Query;",
            &[JValue::Object(&ids_arr)],
        )?;

        let cursor = env
            .call_method(
                &dm,
                "query",
                "(Landroid/app/DownloadManager$Query;)Landroid/database/Cursor;",
                &[JValue::Object(&query)],
            )?
            .l()?;

        let has_row = env
            .call_method(&cursor, "moveToFirst", "()Z", &[])?.z()?;

        if !has_row {
            env.call_method(&cursor, "close", "()V", &[])?;
            std::mem::forget(activity);
            return Ok(DownloadPollResult::InProgress);
        }

        // Colonnes STATUS et LOCAL_URI
        let status_col_name = env.new_string("status")?;
        let status_col = env
            .call_method(
                &cursor,
                "getColumnIndex",
                "(Ljava/lang/String;)I",
                &[JValue::Object(&status_col_name)],
            )?
            .i()?;
        let status = env
            .call_method(&cursor, "getInt", "(I)I", &[JValue::Int(status_col)])?
            .i()?;

        // STATUS_SUCCESSFUL = 8, STATUS_FAILED = 16
        let poll_result = if status == 8 {
            let uri_col_name = env.new_string("local_uri")?;
            let uri_col = env
                .call_method(
                    &cursor,
                    "getColumnIndex",
                    "(Ljava/lang/String;)I",
                    &[JValue::Object(&uri_col_name)],
                )?
                .i()?;
            let uri_obj = env
                .call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(uri_col)])?
                .l()?;
            let path: String = env.get_string(&uri_obj.into())?.into();
            DownloadPollResult::Complete(path)
        } else if status == 16 {
            DownloadPollResult::Failed("DownloadManager STATUS_FAILED".to_string())
        } else {
            DownloadPollResult::InProgress
        };

        env.call_method(&cursor, "close", "()V", &[])?;
        std::mem::forget(activity);
        Ok(poll_result)
    })();

    std::mem::forget(vm);
    result.unwrap_or_else(|e| DownloadPollResult::Failed(e.to_string()))
}

#[cfg(target_os = "android")]
fn android_trigger_install(apk_path: &str) -> Result<(), String> {
    use jni::objects::{JByteArray, JObject, JValue};
    use std::io::Read;

    let local_path = apk_path.trim_start_matches("file://");

    // Lire le contenu de l'APK
    let mut apk_bytes = Vec::new();
    std::fs::File::open(local_path)
        .map_err(|e| format!("Impossible d'ouvrir l'APK : {}", e))?
        .read_to_end(&mut apk_bytes)
        .map_err(|e| format!("Impossible de lire l'APK : {}", e))?;

    let app = bevy::window::ANDROID_APP
        .get()
        .ok_or("Impossible d'obtenir l'AndroidApp")?;

    let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }
        .map_err(|e| format!("JavaVM::from_raw: {}", e))?;

    let result = (|| -> Result<(), jni::errors::Error> {
        let mut env = vm.attach_current_thread_as_daemon()?;

        let activity_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let activity = unsafe { JObject::from_raw(activity_ptr) };

        // Obtenir PackageInstaller via PackageManager
        let pm = env
            .call_method(&activity, "getPackageManager", "()Landroid/content/pm/PackageManager;", &[])?
            .l()?;
        let installer = env
            .call_method(&pm, "getPackageInstaller", "()Landroid/content/pm/PackageInstaller;", &[])?
            .l()?;

        // Créer les paramètres de session
        let params_class = env.find_class("android/content/pm/PackageInstaller$SessionParams")?;
        // MODE_FULL_INSTALL = 1
        let params = env.new_object(params_class, "(I)V", &[JValue::Int(1)])?;

        // Créer la session
        let session_id = env
            .call_method(&installer, "createSession",
                "(Landroid/content/pm/PackageInstaller$SessionParams;)I",
                &[JValue::Object(&params)])?
            .i()?;

        // Ouvrir la session
        let session = env
            .call_method(&installer, "openSession",
                "(I)Landroid/content/pm/PackageInstaller$Session;",
                &[JValue::Int(session_id)])?
            .l()?;

        // Écrire l'APK dans le stream de la session
        let apk_name = env.new_string("monster-battle.apk")?;
        // FLAG_TRUNCATE = 1
        let out_stream = env
            .call_method(&session, "openWrite",
                "(Ljava/lang/String;JJ)Ljava/io/OutputStream;",
                &[JValue::Object(&apk_name), JValue::Long(0), JValue::Long(apk_bytes.len() as i64)])?
            .l()?;

        // Écrire par chunks de 64 Ko
        let chunk_size = 65536usize;
        for chunk in apk_bytes.chunks(chunk_size) {
            let jbytes: JByteArray = env.byte_array_from_slice(chunk)?;
            env.call_method(&out_stream, "write", "([B)V", &[JValue::Object(&jbytes)])?;
        }
        env.call_method(&out_stream, "flush", "()V", &[])?;
        env.call_method(&out_stream, "close", "()V", &[])?;

        // Créer un PendingIntent pour le callback (obligatoire)
        let pkg_name = env
            .call_method(&activity, "getPackageName", "()Ljava/lang/String;", &[])?
            .l()?;
        let pkg_str: String = env.get_string(&pkg_name.into())?.into();
        let action_str = env.new_string(format!("{}.INSTALL_COMPLETE", pkg_str))?;
        let intent = env.new_object(
            "android/content/Intent",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&action_str)],
        )?;
        // FLAG_MUTABLE = 0x02000000 (requis API 31+)
        let pending = env.call_static_method(
            "android/app/PendingIntent",
            "getBroadcast",
            "(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;",
            &[JValue::Object(&activity), JValue::Int(0), JValue::Object(&intent), JValue::Int(0x02000000)],
        )?.l()?;
        let intent_sender = env
            .call_method(&pending, "getIntentSender", "()Landroid/content/IntentSender;", &[])?
            .l()?;

        // Commit (lance l'installation)
        env.call_method(&session, "commit",
            "(Landroid/content/IntentSender;)V",
            &[JValue::Object(&intent_sender)])?;

        std::mem::forget(activity);
        Ok(())
    })();

    std::mem::forget(vm);
    result.map_err(|e| format!("JNI PackageInstaller error: {}", e))
}
