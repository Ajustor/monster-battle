//! Connexion Postgres et migrations.

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Ouvre le pool de connexions et applique les migrations en attente.
pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
