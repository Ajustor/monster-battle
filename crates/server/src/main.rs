use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

use monster_battle_core::Monster;
use monster_battle_network::protocol::NetAction;
use monster_battle_network::{NetMessage, read_message, write_message};

/// Joueur en attente dans une file.
struct QueuedPlayer {
    /// Identifiant unique de la session (socket addr).
    _id: String,
    /// Nom du joueur.
    player_name: String,
    /// Monstre proposé.
    monster: Monster,
    /// Stream WebSocket.
    stream: WebSocketStream<TcpStream>,
}

/// Files d'attente globales du serveur.
struct ServerState {
    /// File d'attente combat.
    combat_queue: Vec<QueuedPlayer>,
    /// File d'attente reproduction.
    breed_queue: Vec<QueuedPlayer>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            combat_queue: Vec::new(),
            breed_queue: Vec::new(),
        }
    }
}

/// Répond à une requête HTTP avec le status de santé.
async fn handle_http(mut stream: TcpStream) {
    // Lire le reste de la requête HTTP (on a déjà peek les premiers octets)
    let mut buf = [0u8; 1024];
    let _ = stream.read(&mut buf).await;
    let body = r#"{"status":"online"}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "7878".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    println!("🎮 Serveur Monster Battle démarré sur {}", addr);
    println!("🌐 WebSocket sur /ws — santé HTTP sur /health");
    println!("   En attente de connexions...");

    let state = Arc::new(Mutex::new(ServerState::new()));

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let peer = peer_addr.to_string();

        // Peek les premiers octets pour distinguer HTTP du reste.
        let mut peek_buf = vec![0u8; 2048];
        let n = match socket.peek(&mut peek_buf).await {
            Ok(n) => n,
            Err(_) => continue,
        };

        let request = String::from_utf8_lossy(&peek_buf[..n]);
        let is_websocket = request.to_ascii_lowercase().contains("upgrade: websocket");
        let is_http =
            request.starts_with("GET") || request.starts_with("HEA") || request.starts_with("POS");

        if is_websocket {
            println!("📡 Connexion WebSocket : {}", peer);
            let state = Arc::clone(&state);
            tokio::spawn(async move {
                match tokio_tungstenite::accept_async(socket).await {
                    Ok(ws_stream) => {
                        if let Err(e) = handle_client(ws_stream, &peer, state).await {
                            // Ignorer les déconnexions propres (health check, etc.)
                            let msg = e.to_string();
                            if !msg.contains("fermée")
                                && !msg.contains("closed")
                                && !msg.contains("reset")
                                && !msg.contains("broken pipe")
                            {
                                eprintln!("❌ Erreur client {} : {}", peer, e);
                            }
                        }
                        println!("👋 Déconnexion : {}", peer);
                    }
                    Err(e) => {
                        eprintln!("❌ WebSocket handshake {} : {}", peer, e);
                    }
                }
            });
        } else if is_http {
            tokio::spawn(handle_http(socket));
        }
    }
}

/// Gère un client qui vient de se connecter via WebSocket.
async fn handle_client(
    mut ws: WebSocketStream<TcpStream>,
    peer: &str,
    state: Arc<Mutex<ServerState>>,
) -> anyhow::Result<()> {
    // Attendre le premier message : Queue { action, monster, player_name }
    let msg = read_message(&mut ws).await?;

    match msg {
        NetMessage::Queue {
            action,
            monster,
            player_name,
        } => {
            println!(
                "📋 {} ({}) s'inscrit pour {:?} avec {}",
                player_name, peer, action, monster.name
            );

            // Confirmer la mise en file
            write_message(&mut ws, &NetMessage::Queued).await?;

            let player = QueuedPlayer {
                _id: peer.to_string(),
                player_name,
                monster,
                stream: ws,
            };

            // Tenter un match
            try_match(player, action, state).await?;
        }
        NetMessage::Ping => {
            write_message(&mut ws, &NetMessage::Pong).await?;
        }
        NetMessage::Disconnect => {
            // Rien à faire
        }
        other => {
            let err = format!("Message inattendu : {:?}", other);
            eprintln!("⚠️  {} : {}", peer, err);
            write_message(&mut ws, &NetMessage::Error(err)).await?;
        }
    }

    Ok(())
}

/// Essaie de jumeler le joueur avec quelqu'un dans la file.
async fn try_match(
    player: QueuedPlayer,
    action: NetAction,
    state: Arc<Mutex<ServerState>>,
) -> anyhow::Result<()> {
    let opponent = {
        let mut guard = state.lock().await;
        let queue = match action {
            NetAction::Combat => &mut guard.combat_queue,
            NetAction::Breed => &mut guard.breed_queue,
        };

        if queue.is_empty() {
            // Pas d'adversaire — mettre en file et attendre
            queue.push(player);
            println!(
                "   ⏳ En attente d'un partenaire... (file: {})",
                queue.len()
            );
            return Ok(());
        }

        // Récupérer le premier joueur en attente
        queue.remove(0)
    };

    // Match trouvé !
    println!(
        "🤝 Match {:?} : {} vs {}",
        action, player.player_name, opponent.player_name
    );

    match action {
        NetAction::Combat => {
            run_combat(player, opponent).await?;
        }
        NetAction::Breed => {
            run_breeding(player, opponent).await?;
        }
    }

    Ok(())
}

/// Exécute un combat PvP entre deux joueurs jumelés.
/// Envoie à chaque joueur le monstre de l'adversaire pour un combat interactif local.
async fn run_combat(mut player_a: QueuedPlayer, mut player_b: QueuedPlayer) -> anyhow::Result<()> {
    // Informer les deux joueurs qu'ils sont jumelés
    write_message(
        &mut player_a.stream,
        &NetMessage::Matched {
            opponent_name: player_b.player_name.clone(),
        },
    )
    .await?;

    write_message(
        &mut player_b.stream,
        &NetMessage::Matched {
            opponent_name: player_a.player_name.clone(),
        },
    )
    .await?;

    // Envoyer à chaque joueur le monstre de l'autre pour combat interactif local
    let opponent_for_a = NetMessage::CombatOpponent {
        opponent_monster: player_b.monster.clone(),
    };
    let opponent_for_b = NetMessage::CombatOpponent {
        opponent_monster: player_a.monster.clone(),
    };

    write_message(&mut player_a.stream, &opponent_for_a).await?;
    write_message(&mut player_b.stream, &opponent_for_b).await?;

    println!(
        "⚔️  Monstres échangés pour combat interactif entre {} et {}",
        player_a.player_name, player_b.player_name
    );

    Ok(())
}

/// Exécute une reproduction entre deux joueurs jumelés.
async fn run_breeding(
    mut player_a: QueuedPlayer,
    mut player_b: QueuedPlayer,
) -> anyhow::Result<()> {
    // Informer les deux joueurs qu'ils sont jumelés
    write_message(
        &mut player_a.stream,
        &NetMessage::Matched {
            opponent_name: player_b.player_name.clone(),
        },
    )
    .await?;

    write_message(
        &mut player_b.stream,
        &NetMessage::Matched {
            opponent_name: player_a.player_name.clone(),
        },
    )
    .await?;

    // Envoyer à chaque joueur le monstre de l'autre
    let partner_for_a = NetMessage::BreedingPartner {
        partner_monster: player_b.monster.clone(),
    };
    let partner_for_b = NetMessage::BreedingPartner {
        partner_monster: player_a.monster.clone(),
    };

    write_message(&mut player_a.stream, &partner_for_a).await?;
    write_message(&mut player_b.stream, &partner_for_b).await?;

    println!(
        "🧬 Données de reproduction échangées entre {} et {}",
        player_a.player_name, player_b.player_name
    );

    Ok(())
}
