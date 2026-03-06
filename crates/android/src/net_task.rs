//! Tâches réseau asynchrones pour le PvP et la reproduction.
//!
//! Utilise un thread dédié avec un runtime tokio pour communiquer
//! avec le serveur relais via WebSocket, et un `mpsc::Receiver` côté
//! Bevy pour recevoir les événements réseau dans le game loop.

use std::sync::{Mutex, mpsc};

use bevy::prelude::*;

use monster_battle_core::Monster;
use monster_battle_core::battle::BattleMessage;
use monster_battle_network::{GameClient, NetAction, NetMessage};
use monster_battle_storage::MonsterStorage;

use crate::connection::resolve_host_jni;

/// Adresse du serveur de jeu.
const SERVER_ADDR: &str = "monster-battle.darthoit.eu";

// ═══════════════════════════════════════════════════════════════════
//  Événements réseau
// ═══════════════════════════════════════════════════════════════════

/// Événements reçus des tâches réseau en arrière-plan.
#[derive(Debug)]
pub enum NetworkEvent {
    /// Mis en file d'attente sur le serveur.
    Queued,
    /// Adversaire / partenaire trouvé.
    Matched { opponent_name: String },
    /// Monstre de l'adversaire PvP reçu — lancer le combat interactif.
    CombatOpponentReceived(Monster),
    /// Résultat d'un tour PvP.
    PvpTurnResult {
        messages: Vec<BattleMessage>,
        player_hp: u32,
        opponent_hp: u32,
        battle_over: bool,
        victory: bool,
        xp_gained: u32,
        loser_died: bool,
        loser_fled: bool,
    },
    /// Les deux joueurs sont prêts — début du prochain tour (PvP).
    PvpNextTurn,
    /// Monstre du partenaire reçu (reproduction).
    BreedingPartnerReceived(Monster),
    /// L'adversaire s'est déconnecté pendant le combat → victoire.
    OpponentDisconnected,
    /// Erreur réseau.
    NetError(String),
}

// ═══════════════════════════════════════════════════════════════════
//  Ressource Bevy
// ═══════════════════════════════════════════════════════════════════

/// Ressource contenant l'état d'une tâche réseau en cours.
#[derive(Resource)]
pub struct NetTask {
    /// Récepteur d'événements réseau (non bloquant via `try_recv`).
    pub rx: Mutex<mpsc::Receiver<NetworkEvent>>,
    /// Émetteur pour envoyer les choix d'attaque PvP au thread réseau.
    pub attack_tx: Option<tokio::sync::mpsc::Sender<usize>>,
    /// UUID du monstre sélectionné pour l'action réseau.
    pub fighter_id: Option<uuid::Uuid>,
    /// Quel type d'action réseau est en cours.
    pub action: NetTaskAction,
}

/// Type d'action réseau en cours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetTaskAction {
    Pvp,
    Breeding,
}

// ═══════════════════════════════════════════════════════════════════
//  Lancement des tâches
// ═══════════════════════════════════════════════════════════════════

/// Lance la tâche réseau PvP (connexion + matchmaking + boucle de combat).
pub fn start_pvp_task(commands: &mut Commands, monster: Monster, fighter_id: uuid::Uuid) {
    let (tx, rx) = mpsc::channel();
    let (attack_tx, mut attack_rx) = tokio::sync::mpsc::channel::<usize>(1);

    let player_name = monster.name.clone();

    std::thread::spawn(move || {
        // Résoudre le DNS via JNI avant de lancer le runtime tokio
        // (getaddrinfo peut échouer sur Android natif)
        let resolved_ip = resolve_host_jni(SERVER_ADDR);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async move {
            let result: Result<(), anyhow::Error> = async {
                let ip = resolved_ip
                    .ok_or_else(|| anyhow::anyhow!("Résolution DNS échouée"))?;

                let client = GameClient::new();
                client.connect_with_resolved_ip(SERVER_ADDR, ip).await?;

                // S'inscrire dans la file de combat
                client
                    .send(&NetMessage::Queue {
                        action: NetAction::Combat,
                        monster: monster.clone(),
                        player_name,
                    })
                    .await?;

                // Attendre la confirmation
                let msg = client.recv().await?;
                match msg {
                    NetMessage::Queued => {}
                    NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                    _ => return Err(anyhow::anyhow!("Réponse inattendue du serveur.")),
                }

                let _ = tx.send(NetworkEvent::Queued);

                // Attendre le match
                let msg = client.recv().await?;
                match msg {
                    NetMessage::Matched { opponent_name } => {
                        let _ = tx.send(NetworkEvent::Matched {
                            opponent_name: opponent_name.clone(),
                        });
                    }
                    NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                    _ => return Err(anyhow::anyhow!("Réponse inattendue du serveur.")),
                }

                // Attendre le monstre adversaire
                let msg = client.recv().await?;
                let opponent_monster = match msg {
                    NetMessage::CombatOpponent { opponent_monster } => opponent_monster,
                    NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                    _ => return Err(anyhow::anyhow!("Données de l'adversaire manquantes.")),
                };

                let _ = tx.send(NetworkEvent::CombatOpponentReceived(opponent_monster));

                // ── Boucle de combat PvP ──
                loop {
                    // Attendre que le joueur local choisisse une attaque
                    let attack_index = match attack_rx.recv().await {
                        Some(idx) => idx,
                        None => break, // Canal fermé → combat annulé
                    };

                    // usize::MAX = signal de forfait (fuite PvP)
                    if attack_index == usize::MAX {
                        client.send(&NetMessage::PvpForfeit).await?;
                    } else if attack_index == usize::MAX - 1 {
                        // Signal PvpReady (joueur a fini de lire les messages du tour)
                        client.send(&NetMessage::PvpReady).await?;

                        // Attendre PvpNextTurn du serveur (ou PvpTurnResult si l'adversaire a fui)
                        loop {
                            let msg = client.recv().await?;
                            match msg {
                                NetMessage::PvpNextTurn => {
                                    let _ = tx.send(NetworkEvent::PvpNextTurn);
                                    break;
                                }
                                NetMessage::PvpTurnResult {
                                    messages,
                                    player_hp,
                                    opponent_hp,
                                    battle_over,
                                    victory,
                                    xp_gained,
                                    loser_died,
                                    loser_fled,
                                } => {
                                    // L'adversaire a fui pendant la phase de lecture
                                    let _ = tx.send(NetworkEvent::PvpTurnResult {
                                        messages,
                                        player_hp,
                                        opponent_hp,
                                        battle_over,
                                        victory,
                                        xp_gained,
                                        loser_died,
                                        loser_fled,
                                    });
                                    if battle_over {
                                        return Ok(());
                                    }
                                    break;
                                }
                                NetMessage::Ping => {
                                    client.send(&NetMessage::Pong).await?;
                                }
                                NetMessage::Error(e) => {
                                    return Err(anyhow::anyhow!("{}", e));
                                }
                                NetMessage::Disconnect => {
                                    let _ = tx.send(NetworkEvent::OpponentDisconnected);
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                        continue;
                    } else {
                        // Envoyer le choix d'attaque au serveur
                        client
                            .send(&NetMessage::PvpAttackChoice { attack_index })
                            .await?;
                    }

                    // Attendre le résultat du tour
                    let msg = match client.recv().await {
                        Ok(msg) => msg,
                        Err(_) => {
                            // Connexion perdue → adversaire déconnecté
                            let _ = tx.send(NetworkEvent::OpponentDisconnected);
                            return Ok(());
                        }
                    };
                    match msg {
                        NetMessage::PvpTurnResult {
                            messages,
                            player_hp,
                            opponent_hp,
                            battle_over,
                            victory,
                            xp_gained,
                            loser_died,
                            loser_fled,
                        } => {
                            let is_over = battle_over;
                            let _ = tx.send(NetworkEvent::PvpTurnResult {
                                messages,
                                player_hp,
                                opponent_hp,
                                battle_over,
                                victory,
                                xp_gained,
                                loser_died,
                                loser_fled,
                            });
                            if is_over {
                                break;
                            }
                        }
                        NetMessage::Disconnect => {
                            let _ = tx.send(NetworkEvent::OpponentDisconnected);
                            return Ok(());
                        }
                        NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                        _ => return Err(anyhow::anyhow!("Réponse inattendue du serveur.")),
                    }
                }

                Ok(())
            }
            .await;

            if let Err(e) = result {
                let _ = tx.send(NetworkEvent::NetError(format!("{}", e)));
            }
        });
    });

    commands.insert_resource(NetTask {
        rx: Mutex::new(rx),
        attack_tx: Some(attack_tx),
        fighter_id: Some(fighter_id),
        action: NetTaskAction::Pvp,
    });
}

/// Lance la tâche réseau de reproduction (connexion + matchmaking).
pub fn start_breeding_task(commands: &mut Commands, monster: Monster, fighter_id: uuid::Uuid) {
    let (tx, rx) = mpsc::channel();

    let player_name = monster.name.clone();

    std::thread::spawn(move || {
        // Résoudre le DNS via JNI avant de lancer le runtime tokio
        let resolved_ip = resolve_host_jni(SERVER_ADDR);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async move {
            let result: Result<Monster, anyhow::Error> = async {
                let ip = resolved_ip
                    .ok_or_else(|| anyhow::anyhow!("Résolution DNS échouée"))?;

                let client = GameClient::new();
                client.connect_with_resolved_ip(SERVER_ADDR, ip).await?;

                // S'inscrire dans la file de reproduction
                client
                    .send(&NetMessage::Queue {
                        action: NetAction::Breed,
                        monster,
                        player_name,
                    })
                    .await?;

                // Attendre la confirmation
                let msg = client.recv().await?;
                match msg {
                    NetMessage::Queued => {}
                    NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                    _ => return Err(anyhow::anyhow!("Réponse inattendue du serveur.")),
                }

                let _ = tx.send(NetworkEvent::Queued);

                // Attendre le match
                let msg = client.recv().await?;
                match msg {
                    NetMessage::Matched { opponent_name } => {
                        let _ = tx.send(NetworkEvent::Matched {
                            opponent_name: opponent_name.clone(),
                        });
                    }
                    NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                    _ => return Err(anyhow::anyhow!("Réponse inattendue du serveur.")),
                }

                // Attendre les données du partenaire
                let msg = client.recv().await?;
                match msg {
                    NetMessage::BreedingPartner { partner_monster } => Ok(partner_monster),
                    NetMessage::Error(e) => Err(anyhow::anyhow!("{}", e)),
                    _ => Err(anyhow::anyhow!("Données du partenaire manquantes.")),
                }
            }
            .await;

            let event = match result {
                Ok(monster) => NetworkEvent::BreedingPartnerReceived(monster),
                Err(e) => NetworkEvent::NetError(format!("{}", e)),
            };
            let _ = tx.send(event);
        });
    });

    commands.insert_resource(NetTask {
        rx: Mutex::new(rx),
        attack_tx: None,
        fighter_id: Some(fighter_id),
        action: NetTaskAction::Breeding,
    });
}

// ═══════════════════════════════════════════════════════════════════
//  Système de polling réseau (Update)
// ═══════════════════════════════════════════════════════════════════

/// Système Bevy qui poll les événements réseau et met à jour l'état du jeu.
pub fn poll_network_events(
    mut commands: Commands,
    net_task: Option<ResMut<NetTask>>,
    mut data: ResMut<crate::game::GameData>,
    mut next_state: ResMut<NextState<crate::game::GameScreen>>,
    state: Res<State<crate::game::GameScreen>>,
) {
    let net_task = match net_task {
        Some(t) => t,
        None => return,
    };

    let event = match net_task.rx.lock().unwrap().try_recv() {
        Ok(event) => event,
        Err(mpsc::TryRecvError::Empty) => return,
        Err(mpsc::TryRecvError::Disconnected) => {
            // La tâche réseau est terminée (sender droppé).
            // Si on est encore sur un écran de recherche, c'est une erreur
            // silencieuse (thread crash / panic) → informer l'utilisateur.
            commands.remove_resource::<NetTask>();
            let current = **state;
            match current {
                crate::game::GameScreen::PvpSearching => {
                    log::error!("❌ Tâche réseau PvP terminée sans événement (thread crash ?)");
                    data.battle_state = None;
                    data.message = Some("Erreur : connexion au serveur impossible.".to_string());
                    next_state.set(crate::game::GameScreen::MainMenu);
                }
                crate::game::GameScreen::BreedingSearching => {
                    log::error!(
                        "❌ Tâche réseau reproduction terminée sans événement (thread crash ?)"
                    );
                    data.remote_monster = None;
                    data.message = Some("Erreur : connexion au serveur impossible.".to_string());
                    next_state.set(crate::game::GameScreen::MainMenu);
                }
                _ => {
                    // Fin normale de la tâche (combat terminé, reproduction finie, etc.)
                }
            }
            return;
        }
    };

    match event {
        NetworkEvent::Queued => {
            // Rien de spécial — on reste sur l'écran de recherche
            log::info!("📡 En file d'attente sur le serveur");
        }
        NetworkEvent::Matched { opponent_name } => {
            log::info!("🎯 Adversaire trouvé : {}", opponent_name);
            data.message = Some(format!("Adversaire trouvé : {} !", opponent_name));
        }
        NetworkEvent::CombatOpponentReceived(opponent_monster) => {
            log::info!("⚔️ Monstre adversaire reçu, lancement du combat PvP");
            data.battle_ui_dirty = true;

            let monsters: Vec<Monster> = match data.storage.list_alive() {
                Ok(m) if !m.is_empty() => m,
                _ => {
                    data.message = Some("Pas de monstre vivant !".to_string());
                    commands.remove_resource::<NetTask>();
                    next_state.set(crate::game::GameScreen::MainMenu);
                    return;
                }
            };

            // Retrouver le monstre sélectionné
            let fighter = net_task
                .fighter_id
                .and_then(|id| monsters.iter().find(|m| m.id == id))
                .or(monsters.first());

            if let Some(fighter) = fighter {
                use monster_battle_core::battle::BattleState;
                let battle = BattleState::new(fighter, &opponent_monster, false);
                data.battle_state = Some(battle);
                next_state.set(crate::game::GameScreen::Battle);
            } else {
                commands.remove_resource::<NetTask>();
                next_state.set(crate::game::GameScreen::MainMenu);
            }
        }
        NetworkEvent::PvpTurnResult {
            messages,
            player_hp,
            opponent_hp,
            battle_over,
            victory,
            xp_gained,
            loser_died,
            loser_fled: _,
        } => {
            use monster_battle_core::battle::BattlePhase;

            if let Some(ref mut battle) = data.battle_state {
                battle.player.current_hp = player_hp;
                battle.opponent.current_hp = opponent_hp;

                if battle_over {
                    if victory {
                        battle.phase = BattlePhase::Victory;
                        battle.xp_gained = xp_gained;
                    } else {
                        battle.phase = BattlePhase::Defeat;
                        battle.loser_died = loser_died;
                    }
                } else {
                    battle.phase = BattlePhase::WaitingForOpponent;
                    battle.attack_menu_index = 0;
                }

                battle.push_messages(messages);
                battle.advance_message();
                data.battle_ui_dirty = true;
            }
        }
        NetworkEvent::PvpNextTurn => {
            use monster_battle_core::battle::BattlePhase;

            if let Some(ref mut battle) = data.battle_state {
                battle.turn += 1;
                battle.phase = BattlePhase::PlayerChooseAttack;
                battle.attack_menu_index = 0;
            }
            data.battle_ui_dirty = true;
        }
        NetworkEvent::OpponentDisconnected => {
            use monster_battle_core::battle::{BattleMessage, BattlePhase, MessageStyle};
            log::info!("Adversaire déconnecté → victoire !");

            if let Some(ref mut battle) = data.battle_state {
                battle.phase = BattlePhase::Victory;
                battle.xp_gained = 50 + (battle.opponent.level * 5);
                battle.push_messages(vec![
                    BattleMessage {
                        text: "L'adversaire s'est déconnecté !".to_string(),
                        style: MessageStyle::Info,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    },
                    BattleMessage {
                        text: "🏆 Victoire par forfait !".to_string(),
                        style: MessageStyle::Victory,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    },
                ]);
                battle.advance_message();
            }
            data.battle_ui_dirty = true;
        }
        NetworkEvent::BreedingPartnerReceived(partner) => {
            log::info!("🧬 Partenaire de reproduction reçu");
            commands.remove_resource::<NetTask>();

            data.remote_monster = Some(partner);
            data.name_input.clear();
            data.message = None;
            next_state.set(crate::game::GameScreen::BreedingNaming);
        }
        NetworkEvent::NetError(e) => {
            log::error!("❌ Erreur réseau : {}", e);
            commands.remove_resource::<NetTask>();

            let current = **state;
            match current {
                crate::game::GameScreen::PvpSearching | crate::game::GameScreen::Battle => {
                    data.battle_state = None;
                    data.message = Some(format!("Erreur réseau : {}", e));
                    next_state.set(crate::game::GameScreen::MainMenu);
                }
                crate::game::GameScreen::BreedingSearching => {
                    data.remote_monster = None;
                    data.message = Some(format!("Erreur réseau : {}", e));
                    next_state.set(crate::game::GameScreen::MainMenu);
                }
                _ => {
                    data.message = Some(format!("Erreur réseau : {}", e));
                }
            }
        }
    }
}
