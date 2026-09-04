//! Stockage serveur des monstres et protocole de synchronisation.
//!
//! Chaque monstre porte un `version` incrémenté à chaque écriture. Un client
//! qui pousse une modification annonce la version sur laquelle il s'est basé ;
//! si elle ne correspond plus, le serveur refuse l'écriture et renvoie sa
//! copie — au client (ou au joueur) de trancher.

use chrono::{DateTime, Utc};
use monster_battle_core::Monster;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Un monstre tel que le serveur le publie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedMonster {
    pub id: Uuid,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
    /// Absent pour un monstre supprimé (pierre tombale de synchronisation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster: Option<Monster>,
}

/// Une modification poussée par un client.
#[derive(Debug, Clone, Deserialize)]
pub struct MonsterChange {
    pub monster: Monster,
    /// Version sur laquelle le client s'est basé. `None` = création.
    #[serde(default)]
    pub base_version: Option<i64>,
}

/// Résultat de l'application d'une modification.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ChangeOutcome {
    /// Écriture acceptée.
    Applied { id: Uuid, version: i64 },
    /// Le client s'est basé sur une version périmée : voici celle du serveur.
    Conflict {
        id: Uuid,
        server: Box<SyncedMonster>,
    },
    /// Le monstre a été refusé par la validation anti-triche.
    Rejected { id: Uuid, problems: Vec<String> },
}

fn row_to_synced(row: &sqlx::postgres::PgRow) -> Result<SyncedMonster, sqlx::Error> {
    let deleted: bool = row.get("deleted");
    let data: serde_json::Value = row.get("data");

    // Un monstre dont le JSON n'est plus déchiffrable par le cœur du jeu ne
    // doit pas faire échouer toute la synchronisation : il est publié comme
    // pierre tombale plutôt que d'interrompre le pull.
    let monster = if deleted {
        None
    } else {
        serde_json::from_value(data).ok()
    };

    Ok(SyncedMonster {
        id: row.get("id"),
        version: row.get("version"),
        updated_at: row.get("updated_at"),
        deleted: deleted || monster.is_none(),
        monster,
    })
}

/// Liste les monstres d'un joueur modifiés après `since`.
///
/// `since` exclusif : un client repasse son dernier `server_time` et ne reçoit
/// que les nouveautés.
pub async fn pull(
    pool: &PgPool,
    owner_id: Uuid,
    since: Option<DateTime<Utc>>,
) -> sqlx::Result<Vec<SyncedMonster>> {
    let rows = sqlx::query(
        "SELECT id, data, version, deleted, updated_at
         FROM monsters
         WHERE owner_id = $1 AND ($2::timestamptz IS NULL OR updated_at > $2)
         ORDER BY updated_at",
    )
    .bind(owner_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    rows.iter().map(row_to_synced).collect()
}

/// Charge un monstre appartenant à un joueur.
pub async fn get(pool: &PgPool, owner_id: Uuid, id: Uuid) -> sqlx::Result<Option<SyncedMonster>> {
    let row = sqlx::query(
        "SELECT id, data, version, deleted, updated_at
         FROM monsters WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;

    row.as_ref().map(row_to_synced).transpose()
}

/// Applique une modification en concurrence optimiste.
pub async fn apply(
    pool: &PgPool,
    owner_id: Uuid,
    change: &MonsterChange,
) -> sqlx::Result<ChangeOutcome> {
    let id = change.monster.id;

    let problems = super::validate::check(&change.monster);
    if !problems.is_empty() {
        return Ok(ChangeOutcome::Rejected { id, problems });
    }

    let data = match serde_json::to_value(&change.monster) {
        Ok(data) => data,
        Err(e) => {
            return Ok(ChangeOutcome::Rejected {
                id,
                problems: vec![format!("monstre non sérialisable : {}", e)],
            });
        }
    };

    let mut tx = pool.begin().await?;

    let existing = sqlx::query("SELECT owner_id, version FROM monsters WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

    match existing {
        None => {
            // Création. `base_version` renseigné signifie que le client croyait
            // le monstre déjà connu du serveur : c'est un conflit, pas une
            // création (le monstre a pu être supprimé entre-temps).
            if change.base_version.unwrap_or(0) != 0 {
                tx.commit().await?;
                return Ok(ChangeOutcome::Conflict {
                    id,
                    server: Box::new(SyncedMonster {
                        id,
                        version: 0,
                        updated_at: Utc::now(),
                        deleted: true,
                        monster: None,
                    }),
                });
            }

            let row = sqlx::query(
                "INSERT INTO monsters (id, owner_id, data) VALUES ($1, $2, $3)
                 RETURNING version",
            )
            .bind(id)
            .bind(owner_id)
            .bind(&data)
            .fetch_one(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok(ChangeOutcome::Applied {
                id,
                version: row.get("version"),
            })
        }
        Some(row) => {
            let current_owner: Uuid = row.get("owner_id");
            let current_version: i64 = row.get("version");

            // Un monstre appartenant à quelqu'un d'autre est traité comme
            // introuvable : on ne révèle rien de la collection d'autrui.
            if current_owner != owner_id {
                tx.commit().await?;
                return Ok(ChangeOutcome::Rejected {
                    id,
                    problems: vec!["ce monstre appartient à un autre joueur".to_string()],
                });
            }

            if change.base_version != Some(current_version) {
                tx.commit().await?;
                let server = get(pool, owner_id, id).await?;
                return Ok(ChangeOutcome::Conflict {
                    id,
                    server: Box::new(
                        server.expect("le monstre existe, on vient de le verrouiller"),
                    ),
                });
            }

            let row = sqlx::query(
                "UPDATE monsters
                 SET data = $2, version = version + 1, deleted = FALSE, updated_at = now()
                 WHERE id = $1
                 RETURNING version",
            )
            .bind(id)
            .bind(&data)
            .fetch_one(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok(ChangeOutcome::Applied {
                id,
                version: row.get("version"),
            })
        }
    }
}

/// Marque un monstre comme supprimé (pierre tombale conservée pour que les
/// autres appareils propagent la suppression).
pub async fn soft_delete(pool: &PgPool, owner_id: Uuid, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query(
        "UPDATE monsters
         SET deleted = TRUE, version = version + 1, updated_at = now()
         WHERE id = $1 AND owner_id = $2 AND deleted = FALSE",
    )
    .bind(id)
    .bind(owner_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Nombre de monstres vivants d'un joueur (pour les quotas et l'arène).
pub async fn count_alive(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<i64> {
    let row = sqlx::query(
        "SELECT count(*) AS n FROM monsters
         WHERE owner_id = $1 AND deleted = FALSE AND (data -> 'died_at') = 'null'::jsonb",
    )
    .bind(owner_id)
    .fetch_one(pool)
    .await?;
    Ok(row.get("n"))
}
