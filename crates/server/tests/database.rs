//! Tests d'intégration sur une vraie base Postgres.
//!
//! Ils ne s'exécutent que si `TEST_DATABASE_URL` est défini ; sinon ils
//! s'ignorent silencieusement, pour que `cargo test` reste utilisable sans
//! base (c'est le cas en CI).
//!
//! Chaque test travaille sur son propre schéma Postgres, ce qui les rend
//! indépendants et permet de les lancer en parallèle.

use std::time::Duration;

use monster_battle_core::Monster;
use monster_battle_core::types::{ElementType, Stats};
use monster_battle_server::auth::oauth::{Provider, ProviderIdentity};
use monster_battle_server::auth::store::{self, DevicePollOutcome};
use monster_battle_server::monsters::store as monsters;
use monster_battle_server::monsters::store::{ChangeOutcome, MonsterChange};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

/// Ouvre un pool sur un schéma dédié au test, migrations appliquées.
///
/// Retourne `None` quand aucune base n'est configurée : l'appelant s'arrête là.
async fn test_pool(name: &str) -> Option<PgPool> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;

    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .expect("connexion à TEST_DATABASE_URL");

    let schema = format!("test_{}", name);
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {} CASCADE", schema))
        .execute(&admin)
        .await
        .unwrap();
    sqlx::query(&format!("CREATE SCHEMA {}", schema))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;

    let pool = PgPoolOptions::new()
        .max_connections(4)
        .after_connect(move |conn, _| {
            let schema = schema.clone();
            Box::pin(async move {
                sqlx::query(&format!("SET search_path TO {}", schema))
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connexion au schéma de test");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    Some(pool)
}

/// Identité OAuth factice.
fn identity(provider: Provider, id: &str, email: Option<&str>) -> ProviderIdentity {
    ProviderIdentity {
        provider,
        provider_user_id: id.to_string(),
        email: email.map(str::to_string),
        display_name: format!("Dresseur {}", id),
        avatar_url: None,
    }
}

fn starter(name: &str) -> Monster {
    Monster::new_starter(
        name.to_string(),
        ElementType::Fire,
        Stats::new(50, 45, 40, 35, 50, 40),
    )
}

// ── Comptes ───────────────────────────────────────────────────────────

#[tokio::test]
async fn une_identite_connue_retrouve_son_compte() {
    let Some(pool) = test_pool("identite_connue").await else {
        return;
    };

    let first = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "42", None))
        .await
        .unwrap();
    let second = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "42", None))
        .await
        .unwrap();

    assert_eq!(first.id, second.id, "la seconde connexion crée un doublon");
}

#[tokio::test]
async fn deux_fournisseurs_partageant_un_email_partagent_le_compte() {
    let Some(pool) = test_pool("meme_email").await else {
        return;
    };

    let email = Some("dresseur@example.com");
    let github = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", email))
        .await
        .unwrap();
    let google = store::upsert_user_from_identity(&pool, &identity(Provider::Google, "2", email))
        .await
        .unwrap();

    assert_eq!(
        github.id, google.id,
        "se reconnecter via un autre fournisseur doit retomber sur le même joueur"
    );
}

#[tokio::test]
async fn deux_identites_sans_email_restent_distinctes() {
    let Some(pool) = test_pool("sans_email").await else {
        return;
    };

    let a = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();
    let b = store::upsert_user_from_identity(&pool, &identity(Provider::Google, "2", None))
        .await
        .unwrap();

    assert_ne!(
        a.id, b.id,
        "sans email commun, rien ne prouve que c'est le même joueur"
    );
}

// ── Sessions ──────────────────────────────────────────────────────────

#[tokio::test]
async fn le_refresh_token_tourne_et_l_ancien_meurt() {
    let Some(pool) = test_pool("rotation").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();
    let ttl = Duration::from_secs(3600);

    let first = store::issue_refresh_token(&pool, user.id, Some("tui"), ttl)
        .await
        .unwrap();

    let (rotated_user, second) = store::rotate_refresh_token(&pool, &first, ttl)
        .await
        .unwrap()
        .expect("le premier jeton est valide");
    assert_eq!(rotated_user, user.id);
    assert_ne!(first, second);

    // Rejouer l'ancien jeton ne doit plus rien donner.
    assert!(
        store::rotate_refresh_token(&pool, &first, ttl)
            .await
            .unwrap()
            .is_none(),
        "un jeton déjà tourné doit être refusé"
    );

    // Le nouveau, lui, fonctionne.
    assert!(
        store::rotate_refresh_token(&pool, &second, ttl)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn un_jeton_revoque_est_refuse() {
    let Some(pool) = test_pool("revocation").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();
    let ttl = Duration::from_secs(3600);

    let token = store::issue_refresh_token(&pool, user.id, None, ttl)
        .await
        .unwrap();

    assert!(store::revoke_refresh_token(&pool, &token).await.unwrap());
    assert!(
        store::rotate_refresh_token(&pool, &token, ttl)
            .await
            .unwrap()
            .is_none()
    );
    // Révoquer deux fois n'est plus une révocation.
    assert!(!store::revoke_refresh_token(&pool, &token).await.unwrap());
}

#[tokio::test]
async fn un_jeton_expire_ne_tourne_pas() {
    let Some(pool) = test_pool("expiration").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    let token = store::issue_refresh_token(&pool, user.id, None, Duration::from_secs(0))
        .await
        .unwrap();

    assert!(
        store::rotate_refresh_token(&pool, &token, Duration::from_secs(3600))
            .await
            .unwrap()
            .is_none()
    );
}

// ── Appairage d'appareil ──────────────────────────────────────────────

#[tokio::test]
async fn le_parcours_d_appairage_complet() {
    let Some(pool) = test_pool("appairage").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    let request = store::create_device_request(&pool, Some("tui"), Duration::from_secs(600))
        .await
        .unwrap();

    // Avant approbation, le client attend.
    assert_eq!(
        store::poll_device_request(&pool, &request.device_code)
            .await
            .unwrap(),
        DevicePollOutcome::Pending
    );

    // Le joueur saisit le code affiché — avec son tiret et en minuscules.
    let request_id = store::find_pending_device_request(&pool, &request.user_code.to_lowercase())
        .await
        .unwrap()
        .expect("le code affiché doit être retrouvé");
    assert!(
        store::approve_device_request(&pool, request_id, user.id)
            .await
            .unwrap()
    );

    // Le client récupère sa session.
    assert_eq!(
        store::poll_device_request(&pool, &request.device_code)
            .await
            .unwrap(),
        DevicePollOutcome::Approved(user.id)
    );

    // Un device_code ne vaut qu'une session : le rejouer ne donne rien.
    assert_eq!(
        store::poll_device_request(&pool, &request.device_code)
            .await
            .unwrap(),
        DevicePollOutcome::Expired
    );
}

#[tokio::test]
async fn un_code_d_appairage_inconnu_est_refuse() {
    let Some(pool) = test_pool("appairage_inconnu").await else {
        return;
    };

    assert_eq!(
        store::poll_device_request(&pool, "un-device-code-inexistant")
            .await
            .unwrap(),
        DevicePollOutcome::Expired
    );
    assert!(
        store::find_pending_device_request(&pool, "ZZZZ-9999")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn un_appairage_expire_n_est_pas_approuvable() {
    let Some(pool) = test_pool("appairage_expire").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    let request = store::create_device_request(&pool, None, Duration::from_secs(0))
        .await
        .unwrap();

    assert!(
        store::find_pending_device_request(&pool, &request.user_code)
            .await
            .unwrap()
            .is_none(),
        "un code expiré ne doit plus être proposable"
    );
    assert_eq!(
        store::poll_device_request(&pool, &request.device_code)
            .await
            .unwrap(),
        DevicePollOutcome::Expired
    );

    let _ = user;
}

// ── États OAuth ───────────────────────────────────────────────────────

#[tokio::test]
async fn un_etat_oauth_ne_sert_qu_une_fois() {
    let Some(pool) = test_pool("etat_oauth").await else {
        return;
    };

    let state = store::create_oauth_state(&pool, "github", None, None, Duration::from_secs(600))
        .await
        .unwrap();

    let consumed = store::consume_oauth_state(&pool, &state)
        .await
        .unwrap()
        .expect("premier usage");
    assert_eq!(consumed.provider, "github");

    assert!(
        store::consume_oauth_state(&pool, &state)
            .await
            .unwrap()
            .is_none(),
        "rejouer un état OAuth doit échouer (anti-CSRF)"
    );
}

// ── Synchronisation des monstres ──────────────────────────────────────

#[tokio::test]
async fn cycle_de_synchronisation_complet() {
    let Some(pool) = test_pool("sync_cycle").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    let mut monster = starter("Flamby");

    // Création.
    let outcome = monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: monster.clone(),
            base_version: None,
        },
    )
    .await
    .unwrap();
    let ChangeOutcome::Applied { version, .. } = outcome else {
        panic!("création refusée : {:?}", outcome);
    };
    assert_eq!(version, 1);

    // Mise à jour basée sur la bonne version.
    monster.happiness = 80;
    let outcome = monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: monster.clone(),
            base_version: Some(1),
        },
    )
    .await
    .unwrap();
    assert!(matches!(outcome, ChangeOutcome::Applied { version: 2, .. }));

    // Un second appareil, resté sur la version 1, est refusé.
    let outcome = monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: monster.clone(),
            base_version: Some(1),
        },
    )
    .await
    .unwrap();
    let ChangeOutcome::Conflict { server, .. } = outcome else {
        panic!(
            "une écriture périmée aurait dû être refusée : {:?}",
            outcome
        );
    };
    assert_eq!(server.version, 2);
    assert_eq!(server.monster.as_ref().unwrap().happiness, 80);

    // Suppression : le monstre devient une pierre tombale.
    assert!(
        monsters::soft_delete(&pool, user.id, monster.id)
            .await
            .unwrap()
    );
    let all = monsters::pull(&pool, user.id, None).await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(all[0].deleted);
    assert!(all[0].monster.is_none());
}

#[tokio::test]
async fn le_pull_incremental_ne_renvoie_que_les_nouveautes() {
    let Some(pool) = test_pool("pull_incremental").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: starter("Premier"),
            base_version: None,
        },
    )
    .await
    .unwrap();

    let checkpoint = chrono::Utc::now();
    // La colonne updated_at a la précision de la microseconde : on laisse
    // le temps avancer pour que la comparaison soit franche.
    tokio::time::sleep(Duration::from_millis(10)).await;

    monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: starter("Second"),
            base_version: None,
        },
    )
    .await
    .unwrap();

    let recent = monsters::pull(&pool, user.id, Some(checkpoint))
        .await
        .unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].monster.as_ref().unwrap().name, "Second");

    assert_eq!(monsters::pull(&pool, user.id, None).await.unwrap().len(), 2);
}

#[tokio::test]
async fn on_ne_peut_pas_ecrire_sur_le_monstre_d_un_autre() {
    let Some(pool) = test_pool("proprietaire").await else {
        return;
    };

    let alice = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();
    let bob = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "2", None))
        .await
        .unwrap();

    let monster = starter("Flamby");
    monsters::apply(
        &pool,
        alice.id,
        &MonsterChange {
            monster: monster.clone(),
            base_version: None,
        },
    )
    .await
    .unwrap();

    let outcome = monsters::apply(
        &pool,
        bob.id,
        &MonsterChange {
            monster: monster.clone(),
            base_version: Some(1),
        },
    )
    .await
    .unwrap();
    assert!(
        matches!(outcome, ChangeOutcome::Rejected { .. }),
        "Bob ne doit pas pouvoir écrire le monstre d'Alice : {:?}",
        outcome
    );

    // Et il ne le voit pas non plus.
    assert!(
        monsters::get(&pool, bob.id, monster.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        !monsters::soft_delete(&pool, bob.id, monster.id)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn un_monstre_trafique_est_refuse() {
    let Some(pool) = test_pool("anti_triche").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    let mut monster = starter("Tricheur");
    monster.level = 9999;
    monster.base_stats.attack = 60000;

    let outcome = monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster,
            base_version: None,
        },
    )
    .await
    .unwrap();

    let ChangeOutcome::Rejected { problems, .. } = outcome else {
        panic!("un monstre trafiqué aurait dû être refusé : {:?}", outcome);
    };
    assert!(problems.len() >= 2, "problèmes détectés : {:?}", problems);

    // Rien n'a été écrit.
    assert!(
        monsters::pull(&pool, user.id, None)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn le_compteur_de_monstres_vivants_ignore_les_morts() {
    let Some(pool) = test_pool("compteur_vivants").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: starter("Vivant"),
            base_version: None,
        },
    )
    .await
    .unwrap();

    let mut mort = starter("Défunt");
    mort.current_hp = 0;
    mort.died_at = Some(chrono::Utc::now());
    monsters::apply(
        &pool,
        user.id,
        &MonsterChange {
            monster: mort,
            base_version: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(monsters::count_alive(&pool, user.id).await.unwrap(), 1);
}

#[tokio::test]
async fn la_purge_supprime_les_etats_expires() {
    let Some(pool) = test_pool("purge").await else {
        return;
    };

    let state = store::create_oauth_state(&pool, "github", None, None, Duration::from_secs(0))
        .await
        .unwrap();

    store::purge_expired(&pool).await.unwrap();

    assert!(
        store::consume_oauth_state(&pool, &state)
            .await
            .unwrap()
            .is_none()
    );
}

/// Un identifiant de monstre inconnu ne doit rien faire remonter.
#[tokio::test]
async fn un_monstre_inconnu_est_introuvable() {
    let Some(pool) = test_pool("inconnu").await else {
        return;
    };

    let user = store::upsert_user_from_identity(&pool, &identity(Provider::GitHub, "1", None))
        .await
        .unwrap();

    assert!(
        monsters::get(&pool, user.id, Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}
