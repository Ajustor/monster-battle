use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

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
    /// Stream TCP.
    stream: TcpStream,
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

/// Mini serveur HTTP qui répond 200 OK sur toute requête (health check).
async fn run_health_server(addr: String) {
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "⚠️  Impossible de démarrer le health check sur {} : {}",
                addr, e
            );
            return;
        }
    };

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = r#"{"status":"online"}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "7878".to_string());
    let health_port = std::env::var("HEALTH_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Route HTTP /health pour vérifier que le serveur est up
    let health_addr = format!("0.0.0.0:{}", health_port);
    tokio::spawn(run_health_server(health_addr));

    let listener = TcpListener::bind(&addr).await?;
    println!("🎮 Serveur Monster Battle démarré sur {}", addr);
    println!("🩺 Route santé HTTP sur le port {}", health_port);
    println!("   En attente de connexions...");

    let state = Arc::new(Mutex::new(ServerState::new()));

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let peer = peer_addr.to_string();
        println!("📡 Connexion entrante : {}", peer);

        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, &peer, state).await {
                eprintln!("❌ Erreur client {} : {}", peer, e);
            }
            println!("👋 Déconnexion : {}", peer);
        });
    }
}

/// Gère un client qui vient de se connecter.
async fn handle_client(
    mut stream: TcpStream,
    peer: &str,
    state: Arc<Mutex<ServerState>>,
) -> anyhow::Result<()> {
    // Attendre le premier message : Queue { action, monster, player_name }
    let msg = read_message(&mut stream).await?;

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
            write_message(&mut stream, &NetMessage::Queued).await?;

            let player = QueuedPlayer {
                _id: peer.to_string(),
                player_name,
                monster,
                stream,
            };

            // Tenter un match
            try_match(player, action, state).await?;
        }
        NetMessage::Ping => {
            write_message(&mut stream, &NetMessage::Pong).await?;
        }
        NetMessage::Disconnect => {
            // Rien à faire
        }
        other => {
            let err = format!("Message inattendu : {:?}", other);
            eprintln!("⚠️  {} : {}", peer, err);
            write_message(&mut stream, &NetMessage::Error(err)).await?;
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

    // Préparer les monstres pour le combat
    let mut monster_a = player_a.monster.clone();
    let mut monster_b = player_b.monster.clone();
    monster_a.current_hp = monster_a.max_hp();
    monster_b.current_hp = monster_b.max_hp();

    // Lancer le combat
    let result = monster_battle_core::combat::fight(&mut monster_a, &mut monster_b)
        .map_err(|e| anyhow::anyhow!(e))?;

    let log: Vec<String> = result.log.iter().map(|e| e.describe()).collect();

    println!(
        "⚔️  Résultat : vainqueur={} perdant={} mort={}",
        result.winner_id, result.loser_id, result.loser_died
    );

    // Envoyer les résultats aux deux joueurs
    // Joueur A reçoit son monstre mis à jour
    let result_for_a = NetMessage::CombatResult {
        winner_id: result.winner_id.to_string(),
        loser_id: result.loser_id.to_string(),
        loser_died: result.loser_died,
        log: log.clone(),
        updated_monster: monster_a,
    };

    // Joueur B reçoit son monstre mis à jour
    let result_for_b = NetMessage::CombatResult {
        winner_id: result.winner_id.to_string(),
        loser_id: result.loser_id.to_string(),
        loser_died: result.loser_died,
        log,
        updated_monster: monster_b,
    };

    write_message(&mut player_a.stream, &result_for_a).await?;
    write_message(&mut player_b.stream, &result_for_b).await?;

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
