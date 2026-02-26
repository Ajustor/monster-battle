use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, oneshot};
use tokio_tungstenite::WebSocketStream;

use monster_battle_core::Monster;
use monster_battle_core::battle::{BattleMessage, BattlePhase, BattleState, MessageStyle};
use monster_battle_network::protocol::NetAction;
use monster_battle_network::{NetMessage, read_message, write_message};

/// Entrée dans la file d'attente — le WebSocket n'est PAS stocké ici.
/// Le handler garde le WebSocket et écoute la déconnexion.
struct QueueEntry {
    /// Identifiant unique de la session (socket addr).
    id: String,
    /// Nom du joueur.
    player_name: String,
    /// Monstre proposé.
    monster: Monster,
    /// Canal pour signaler qu'un match a été trouvé.
    match_tx: oneshot::Sender<MatchInfo>,
}

/// Données envoyées au joueur en attente quand un match est trouvé.
struct MatchInfo {
    /// Nom de l'adversaire.
    opponent_name: String,
    /// Monstre de l'adversaire.
    opponent_monster: Monster,
    /// Pour le combat : canal pour transférer le WebSocket du joueur en attente
    /// vers le joueur hôte qui exécute la boucle de combat.
    ws_transfer_tx: Option<oneshot::Sender<WebSocketStream<TcpStream>>>,
}

/// Files d'attente globales du serveur.
struct ServerState {
    /// File d'attente combat.
    combat_queue: Vec<QueueEntry>,
    /// File d'attente reproduction.
    breed_queue: Vec<QueueEntry>,
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
    let body = format!(
        r#"{{"status":"online","version":"{}"}}"#,
        env!("CARGO_PKG_VERSION")
    );
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

            // ── Chercher un adversaire dans la file ──
            // Boucle pour ignorer les entrées dont le handler est mort (joueur déconnecté).
            let opponent_entry = loop {
                let entry = {
                    let mut guard = state.lock().await;
                    let queue = match action {
                        NetAction::Combat => &mut guard.combat_queue,
                        NetAction::Breed => &mut guard.breed_queue,
                    };

                    if queue.is_empty() {
                        None
                    } else {
                        Some(queue.remove(0))
                    }
                };

                match entry {
                    Some(e) => {
                        // Vérifier que le handler du joueur est encore vivant
                        // (si le Receiver a été droppé, le joueur s'est déconnecté)
                        if e.match_tx.is_closed() {
                            println!(
                                "   ⚠️  {} (en attente) déconnecté, nettoyage…",
                                e.player_name
                            );
                            continue; // essayer le suivant
                        }
                        break Some(e);
                    }
                    None => break None,
                }
            };

            if let Some(entry) = opponent_entry {
                // ── Match trouvé ! Nous sommes le « nouvel arrivant » (hôte). ──

                // Pour le combat : canal de transfert du WebSocket adverse
                let (ws_transfer_tx, ws_transfer_rx) = oneshot::channel();
                let ws_transfer = match action {
                    NetAction::Combat => Some(ws_transfer_tx),
                    NetAction::Breed => {
                        drop(ws_transfer_tx);
                        None
                    }
                };

                // Signaler le joueur en attente via son canal
                if entry
                    .match_tx
                    .send(MatchInfo {
                        opponent_name: player_name.clone(),
                        opponent_monster: monster.clone(),
                        ws_transfer_tx: ws_transfer,
                    })
                    .is_err()
                {
                    // Le handler adverse s'est déconnecté entre le is_closed() et le send()
                    // Pas de match, on se remet en file et on attend
                    return handle_wait_in_queue(ws, peer, &player_name, &monster, action, &state)
                        .await;
                }

                println!(
                    "🤝 Match {:?} : {} vs {}",
                    action, player_name, entry.player_name
                );

                // Envoyer Matched à notre joueur
                write_message(
                    &mut ws,
                    &NetMessage::Matched {
                        opponent_name: entry.player_name.clone(),
                    },
                )
                .await?;

                match action {
                    NetAction::Breed => {
                        // Envoyer le monstre du partenaire
                        write_message(
                            &mut ws,
                            &NetMessage::BreedingPartner {
                                partner_monster: entry.monster,
                            },
                        )
                        .await?;
                        println!("🧬 Reproduction : {} ← données envoyées", player_name);
                    }
                    NetAction::Combat => {
                        // Envoyer le monstre adverse
                        write_message(
                            &mut ws,
                            &NetMessage::CombatOpponent {
                                opponent_monster: entry.monster.clone(),
                            },
                        )
                        .await?;

                        // Recevoir le WebSocket de l'adversaire (transféré par son handler)
                        let opponent_ws = ws_transfer_rx.await.map_err(|_| {
                            anyhow::anyhow!("L'adversaire s'est déconnecté avant le combat")
                        })?;

                        // Lancer la boucle de combat avec les deux WebSockets
                        run_combat_loop(
                            ws,
                            opponent_ws,
                            &monster,
                            &entry.monster,
                            &player_name,
                            &entry.player_name,
                        )
                        .await?;
                    }
                }
            } else {
                // ── Pas d'adversaire — on se met en file et on attend. ──
                handle_wait_in_queue(ws, peer, &player_name, &monster, action, &state).await?;
            }
        }
        NetMessage::Ping => {
            write_message(&mut ws, &NetMessage::Pong).await?;
        }
        NetMessage::VersionCheck => {
            write_message(
                &mut ws,
                &NetMessage::VersionInfo {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            )
            .await?;
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

/// Met le joueur en file d'attente et garde le WebSocket vivant
/// pour détecter une déconnexion. Attend soit un match soit une déco.
async fn handle_wait_in_queue(
    mut ws: WebSocketStream<TcpStream>,
    peer: &str,
    player_name: &str,
    monster: &Monster,
    action: NetAction,
    state: &Arc<Mutex<ServerState>>,
) -> anyhow::Result<()> {
    let (match_tx, match_rx) = oneshot::channel();

    {
        let mut guard = state.lock().await;
        let queue = match action {
            NetAction::Combat => &mut guard.combat_queue,
            NetAction::Breed => &mut guard.breed_queue,
        };
        queue.push(QueueEntry {
            id: peer.to_string(),
            player_name: player_name.to_string(),
            monster: monster.clone(),
            match_tx,
        });
        println!(
            "   ⏳ {} en attente d'un partenaire... (file {:?}: {})",
            player_name,
            action,
            queue.len()
        );
    }

    // Attendre soit un signal de match, soit une déconnexion du joueur.
    // `biased` donne la priorité au match (si les deux arrivent en même temps).
    tokio::select! {
        biased;

        result = match_rx => {
            match result {
                Ok(info) => {
                    println!(
                        "🤝 {} matché avec {} (depuis la file)",
                        player_name, info.opponent_name
                    );

                    // Envoyer Matched
                    write_message(
                        &mut ws,
                        &NetMessage::Matched {
                            opponent_name: info.opponent_name.clone(),
                        },
                    )
                    .await?;

                    match action {
                        NetAction::Breed => {
                            write_message(
                                &mut ws,
                                &NetMessage::BreedingPartner {
                                    partner_monster: info.opponent_monster,
                                },
                            )
                            .await?;
                            println!(
                                "🧬 Reproduction : {} ← données envoyées",
                                player_name
                            );
                        }
                        NetAction::Combat => {
                            write_message(
                                &mut ws,
                                &NetMessage::CombatOpponent {
                                    opponent_monster: info.opponent_monster,
                                },
                            )
                            .await?;

                            // Transférer notre WebSocket au joueur hôte
                            // pour qu'il puisse exécuter la boucle de combat
                            if let Some(tx) = info.ws_transfer_tx {
                                // Le send peut échouer si le hôte s'est déconnecté
                                let _ = tx.send(ws);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Le Sender a été droppé sans envoyer — cas improbable
                    // (l'entrée a été poppée mais le send a échoué côté hôte)
                    // On ne fait rien, le handler va se terminer.
                }
            }
        }

        result = read_message(&mut ws) => {
            // Le joueur a envoyé un message ou la connexion est tombée.
            // Dans tous les cas, le retirer de la file.
            {
                let mut guard = state.lock().await;
                let queue = match action {
                    NetAction::Combat => &mut guard.combat_queue,
                    NetAction::Breed => &mut guard.breed_queue,
                };
                queue.retain(|e| e.id != peer);
            }

            match result {
                Ok(NetMessage::Disconnect) | Ok(NetMessage::CancelQueue) => {
                    println!(
                        "   ↩️  {} ({}) a annulé la file {:?}",
                        player_name, peer, action
                    );
                }
                Ok(NetMessage::Ping) => {
                    let _ = write_message(&mut ws, &NetMessage::Pong).await;
                }
                Ok(_) => {
                    println!(
                        "   ⚠️  {} ({}) message inattendu en file, nettoyage",
                        player_name, peer
                    );
                }
                Err(_) => {
                    println!(
                        "   ⚠️  {} ({}) déconnecté de la file {:?}",
                        player_name, peer, action
                    );
                }
            }
        }
    }

    Ok(())
}

/// Exécute la boucle de combat PvP interactif entre deux joueurs déjà jumelés.
/// Les messages d'intro (Matched, CombatOpponent) ont déjà été envoyés.
/// `player_a` = joueur hôte (nouvel arrivant), `player_b` = joueur invité (était en file).
async fn run_combat_loop(
    mut ws_a: WebSocketStream<TcpStream>,
    mut ws_b: WebSocketStream<TcpStream>,
    monster_a: &Monster,
    monster_b: &Monster,
    name_a: &str,
    name_b: &str,
) -> anyhow::Result<()> {
    // Créer le BattleState côté serveur (player_a = "player", player_b = "opponent")
    let mut battle = BattleState::new(monster_a, monster_b, false);

    // Passer l'intro (les clients affichent l'intro localement)
    while battle.phase == BattlePhase::Intro {
        if !battle.advance_message() {
            break;
        }
    }
    // Drainer les messages d'intro (déjà affichés côté client)
    battle.drain_messages();
    battle.current_message = None;

    println!("⚔️  Combat PvP interactif entre {} et {}", name_a, name_b);

    // Boucle de combat tour par tour
    loop {
        // Attendre les choix d'attaque des deux joueurs en parallèle
        enum PlayerChoice {
            Attack(usize),
            Forfeit,
        }

        async fn wait_for_player_choice(
            ws: &mut WebSocketStream<TcpStream>,
        ) -> anyhow::Result<PlayerChoice> {
            loop {
                let msg = read_message(ws).await?;
                match msg {
                    NetMessage::PvpAttackChoice { attack_index } => {
                        return Ok(PlayerChoice::Attack(attack_index));
                    }
                    NetMessage::PvpForfeit => return Ok(PlayerChoice::Forfeit),
                    NetMessage::PvpReady => {
                        // Ignorer un PvpReady tardif pendant la phase de choix
                    }
                    NetMessage::Ping => {
                        write_message(ws, &NetMessage::Pong).await?;
                    }
                    NetMessage::Disconnect => {
                        return Err(anyhow::anyhow!("Joueur déconnecté pendant le combat"));
                    }
                    _ => {}
                }
            }
        }

        let (choice_a, choice_b) = tokio::try_join!(
            wait_for_player_choice(&mut ws_a),
            wait_for_player_choice(&mut ws_b),
        )?;

        // Gérer les forfeits
        let a_forfeited = matches!(choice_a, PlayerChoice::Forfeit);
        let b_forfeited = matches!(choice_b, PlayerChoice::Forfeit);

        if a_forfeited || b_forfeited {
            let a_wins = b_forfeited && !a_forfeited;
            let xp = if a_wins {
                50 + (battle.opponent.level * 5)
            } else {
                50 + (battle.player.level * 5)
            };

            let forfeit_name = if a_forfeited { name_a } else { name_b };
            println!("🏳️  {} a fui le combat PvP !", forfeit_name);

            let forfeit_msg = BattleMessage {
                text: format!("🏳️ {} a fui le combat !", forfeit_name),
                style: if a_wins {
                    MessageStyle::Victory
                } else {
                    MessageStyle::Defeat
                },
                player_hp: None,
                opponent_hp: None,
                anim_type: None,
            };
            let flipped_forfeit = forfeit_msg.flip_perspective();

            let result_a = NetMessage::PvpTurnResult {
                messages: vec![forfeit_msg],
                player_hp: battle.player.current_hp,
                opponent_hp: battle.opponent.current_hp,
                battle_over: true,
                victory: a_wins,
                xp_gained: if a_wins { xp } else { 0 },
                loser_died: false,
                loser_fled: true,
            };
            let result_b = NetMessage::PvpTurnResult {
                messages: vec![flipped_forfeit],
                player_hp: battle.opponent.current_hp,
                opponent_hp: battle.player.current_hp,
                battle_over: true,
                victory: !a_wins,
                xp_gained: if !a_wins { xp } else { 0 },
                loser_died: false,
                loser_fled: true,
            };

            write_message(&mut ws_a, &result_a).await?;
            write_message(&mut ws_b, &result_b).await?;
            break;
        }

        let attack_a = match choice_a {
            PlayerChoice::Attack(idx) => idx,
            _ => 0,
        };
        let attack_b = match choice_b {
            PlayerChoice::Attack(idx) => idx,
            _ => 0,
        };

        // Valider les indices d'attaque
        let max_a = battle.player.attacks.len();
        let max_b = battle.opponent.attacks.len();
        let attack_a = if attack_a >= max_a { 0 } else { attack_a };
        let attack_b = if attack_b >= max_b { 0 } else { attack_b };

        println!(
            "   Tour {} : {} attaque #{}, {} attaque #{}",
            battle.turn, name_a, attack_a, name_b, attack_b
        );

        // Résoudre le tour (player_a = player, player_b = opponent)
        battle.pvp_attack(attack_a, attack_b);

        // Collecter les messages générés
        let messages = battle.drain_messages();

        let battle_over = matches!(battle.phase, BattlePhase::Victory | BattlePhase::Defeat);
        let player_a_wins = battle.phase == BattlePhase::Victory;

        // Construire les messages pour chaque joueur
        let mut msgs_a = messages.clone();
        let mut msgs_b: Vec<BattleMessage> =
            messages.iter().map(|m| m.flip_perspective()).collect();

        // Ajouter les messages de fin personnalisés pour chaque joueur
        if battle_over {
            let xp = battle.xp_gained;

            let victory_msg = BattleMessage {
                text: "🏆 Vous avez gagné le combat !".to_string(),
                style: MessageStyle::Victory,
                player_hp: None,
                opponent_hp: None,
                anim_type: None,
            };
            let xp_msg = BattleMessage {
                text: format!("📖 +{} XP !", xp),
                style: MessageStyle::Info,
                player_hp: None,
                opponent_hp: None,
                anim_type: None,
            };
            let defeat_msg = BattleMessage {
                text: "Vous avez perdu le combat...".to_string(),
                style: MessageStyle::Defeat,
                player_hp: None,
                opponent_hp: None,
                anim_type: None,
            };

            if player_a_wins {
                msgs_a.push(victory_msg);
                msgs_a.push(xp_msg);
                msgs_b.push(defeat_msg);
            } else {
                msgs_b.push(victory_msg);
                msgs_b.push(xp_msg);
                msgs_a.push(defeat_msg);
            }
        }

        // Envoyer les messages à player_a (perspective directe)
        let result_a = NetMessage::PvpTurnResult {
            messages: msgs_a,
            player_hp: battle.player.current_hp,
            opponent_hp: battle.opponent.current_hp,
            battle_over,
            victory: player_a_wins,
            xp_gained: if player_a_wins { battle.xp_gained } else { 0 },
            loser_died: battle.loser_died,
            loser_fled: false,
        };

        // Envoyer les messages à player_b (perspective inversée)
        let result_b = NetMessage::PvpTurnResult {
            messages: msgs_b,
            player_hp: battle.opponent.current_hp,
            opponent_hp: battle.player.current_hp,
            battle_over,
            victory: !player_a_wins,
            xp_gained: if !player_a_wins { battle.xp_gained } else { 0 },
            loser_died: battle.loser_died,
            loser_fled: false,
        };

        write_message(&mut ws_a, &result_a).await?;
        write_message(&mut ws_b, &result_b).await?;

        if battle_over {
            println!(
                "⚔️  Combat terminé : {} a gagné !",
                if player_a_wins { name_a } else { name_b },
            );
            break;
        }

        // Attendre que les deux joueurs aient fini de lire les messages du tour
        async fn wait_for_player_ready(ws: &mut WebSocketStream<TcpStream>) -> anyhow::Result<()> {
            loop {
                let msg = read_message(ws).await?;
                match msg {
                    NetMessage::PvpReady => return Ok(()),
                    NetMessage::Ping => {
                        write_message(ws, &NetMessage::Pong).await?;
                    }
                    NetMessage::Disconnect => {
                        return Err(anyhow::anyhow!("Joueur déconnecté pendant le combat"));
                    }
                    _ => {}
                }
            }
        }

        tokio::try_join!(
            wait_for_player_ready(&mut ws_a),
            wait_for_player_ready(&mut ws_b),
        )?;

        // Les deux joueurs sont prêts → envoyer le signal de nouveau tour
        write_message(&mut ws_a, &NetMessage::PvpNextTurn).await?;
        write_message(&mut ws_b, &NetMessage::PvpNextTurn).await?;
    }

    Ok(())
}
