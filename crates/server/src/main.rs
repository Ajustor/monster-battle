//! Serveur Monster Battle.
//!
//! Un seul port sert deux protocoles : le relais WebSocket historique (combats
//! PvP, reproduction) et l'API HTTP des comptes et de la synchronisation. Les
//! premiers octets de la connexion suffisent à les distinguer, ce qui évite
//! d'exiger deux ports ouverts en hébergement.
//!
//! Sans `DATABASE_URL`, le serveur démarre en relais seul : le déploiement
//! existant continue de fonctionner sans base de données.

use std::sync::Arc;

use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tower::ServiceExt;

use monster_battle_server::config::Config;
use monster_battle_server::{api, auth, db, relay};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::from_env()?);
    let addr = format!("0.0.0.0:{}", config.port);

    let listener = TcpListener::bind(&addr).await?;
    println!("🎮 Serveur Monster Battle démarré sur {}", addr);

    // L'API n'est montée que si une base est configurée.
    let app = match &config.database_url {
        Some(database_url) => {
            let pool = db::connect(database_url).await?;
            println!("🗄️  Base de données connectée, migrations à jour");

            let state = api::AppState {
                config: Arc::clone(&config),
                db: pool.clone(),
                http: reqwest::Client::builder()
                    .user_agent(concat!("monster-battle/", env!("CARGO_PKG_VERSION")))
                    .build()?,
            };

            spawn_purge_task(pool);

            println!("🔐 API comptes + synchronisation sur /api/v1");
            let providers = [
                ("GitHub", config.github.is_some()),
                ("Google", config.google.is_some()),
            ];
            for (name, configured) in providers {
                println!(
                    "   {} connexion {}",
                    if configured { "✅" } else { "⚠️ " },
                    if configured {
                        name.to_string()
                    } else {
                        format!("{} (non configurée)", name)
                    }
                );
            }

            Some(api::router(state))
        }
        None => {
            println!("⚠️  DATABASE_URL absent : mode relais seul, sans comptes");
            None
        }
    };

    println!("🌐 WebSocket de combat sur /ws — santé HTTP sur /health");
    println!("   En attente de connexions...");

    let relay_state = relay::new_state();

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let peer = peer_addr.to_string();

        // Peek les premiers octets pour distinguer HTTP du WebSocket.
        let mut peek_buf = vec![0u8; 2048];
        let n = match socket.peek(&mut peek_buf).await {
            Ok(0) => continue,
            Ok(n) => n,
            Err(_) => continue,
        };

        let request = String::from_utf8_lossy(&peek_buf[..n]);
        let is_websocket = request.to_ascii_lowercase().contains("upgrade: websocket");
        let is_http = request.starts_with("GET")
            || request.starts_with("HEA")
            || request.starts_with("POS")
            || request.starts_with("DEL")
            || request.starts_with("OPT")
            || request.starts_with("PUT")
            || request.starts_with("PAT");

        if is_websocket {
            println!("📡 Connexion WebSocket : {}", peer);
            let state = Arc::clone(&relay_state);
            tokio::spawn(relay::serve_connection(socket, peer, state));
        } else if is_http {
            let Some(app) = app.clone() else {
                // Mode relais seul : on répond quand même au health check.
                tokio::spawn(serve_minimal_health(socket));
                continue;
            };

            tokio::spawn(async move {
                let service =
                    hyper::service::service_fn(move |request| app.clone().oneshot(request));

                if let Err(e) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(socket), service)
                    .await
                {
                    let msg = e.to_string();
                    if !msg.contains("closed") && !msg.contains("reset") {
                        eprintln!("❌ Erreur HTTP {} : {}", peer, msg);
                    }
                }
            });
        }
    }
}

/// Réponse de santé minimale quand l'API n'est pas montée.
async fn serve_minimal_health(mut socket: tokio::net::TcpStream) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = [0u8; 1024];
    let _ = socket.read(&mut buf).await;

    let body = format!(
        r#"{{"status":"online","version":"{}","accounts":false}}"#,
        env!("CARGO_PKG_VERSION")
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = socket.write_all(response.as_bytes()).await;
}

/// Purge périodiquement les états OAuth et appairages expirés.
fn spawn_purge_task(pool: sqlx::PgPool) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60 * 60));
        loop {
            ticker.tick().await;
            if let Err(e) = auth::store::purge_expired(&pool).await {
                eprintln!("⚠️  Purge des jetons expirés : {}", e);
            }
        }
    });
}
