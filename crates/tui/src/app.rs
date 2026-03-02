use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, style::Color};
use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use tachyonfx::fx::Direction;
use tachyonfx::{EffectTimer, Interpolation, Shader, fx};

use uuid::Uuid;

use monster_battle_core::Monster;
use monster_battle_core::battle::{BattleMessage, BattlePhase, BattleState, MessageStyle};
use monster_battle_network::{
    GameClient, NetAction, NetMessage, check_server_health, check_server_version,
};
use monster_battle_storage::{LocalStorage, MonsterStorage};

use crate::screens;
use crate::screens::Screen;
use crate::screens::SelectMonsterTarget;
use crate::screens::breeding::BreedPhase;
use crate::screens::pvp::PvpPhase;

/// Événements réseau reçus des tâches en arrière-plan.
enum NetworkEvent {
    /// Mis en file d'attente sur le serveur.
    Queued,
    /// Adversaire trouvé.
    Matched { opponent_name: String },
    /// Monstre de l'adversaire PvP reçu — lancer le combat interactif.
    CombatOpponentReceived(Monster),
    /// Résultat d'un tour PvP (messages du serveur).
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
    /// L'adversaire s'est déconnecté pendant le combat → victoire.
    OpponentDisconnected,
    /// Monstre du partenaire reçu (reproduction).
    BreedingPartnerReceived(Monster),
    /// Erreur réseau.
    NetError(String),
}

/// Statut de la connexion au serveur relais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    /// Pas encore vérifié.
    Unknown,
    /// Serveur joignable.
    Online,
    /// Serveur injoignable.
    Offline,
}

/// État global de l'application.
pub struct App {
    pub storage: LocalStorage,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub menu_index: usize,
    pub message: Option<String>,

    /// Champ de saisie pour le nom du monstre.
    pub name_input: String,
    /// Clignotement du curseur dans le champ de saisie.
    pub name_input_blink: bool,
    /// Instant du dernier blink.
    blink_timer: Instant,

    /// Offset de scroll pour le log de reproduction.
    pub scroll_offset: usize,

    // --- Réseau ---
    /// Adresse du serveur relais (ex: "monster-battle.darthoit.eu").
    pub server_address: String,

    // --- Reproduction ---
    /// Log du résultat de reproduction.
    pub breeding_log: Vec<String>,
    /// Monstre du partenaire distant (reçu via réseau).
    pub remote_monster: Option<Monster>,

    /// État du combat interactif en cours (style Pokémon).
    pub battle_state: Option<BattleState>,
    /// UUID du monstre actuellement en combat (pour le retrouver après).
    fighter_id: Option<Uuid>,

    /// Statut de connexion au serveur relais.
    pub server_status: ServerStatus,
    /// Récepteur pour les mises à jour du statut serveur.
    status_rx: Option<mpsc::Receiver<ServerStatus>>,

    /// Afficher la modale de mise à jour ?
    pub show_update_modal: bool,
    /// Version du serveur (si différente du client).
    pub server_version: Option<String>,
    /// Récepteur pour le résultat du check de version.
    version_rx: Option<mpsc::Receiver<Option<String>>>,

    /// Runtime tokio pour les opérations réseau.
    tokio_rt: tokio::runtime::Runtime,
    /// Récepteur pour les événements réseau en arrière-plan.
    net_rx: Option<mpsc::Receiver<NetworkEvent>>,
    /// Handle de la tâche réseau en cours.
    net_task: Option<tokio::task::JoinHandle<()>>,
    /// Canal pour envoyer les choix d'attaque PvP à la tâche réseau.
    pvp_attack_tx: Option<tokio::sync::mpsc::Sender<usize>>,

    /// Index de sélection du monstre (écran SelectMonster).
    pub monster_select_index: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let data_dir = dirs_data_dir().join("monster-battle").join("monsters");
        let storage = LocalStorage::new(&data_dir)?;

        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            storage,
            current_screen: Screen::MainMenu,
            should_quit: false,
            menu_index: 0,
            message: None,
            name_input: String::new(),
            name_input_blink: true,
            blink_timer: Instant::now(),
            scroll_offset: 0,
            server_address: std::env::var("MONSTER_SERVER")
                .unwrap_or_else(|_| "monster-battle.darthoit.eu".to_string()),
            breeding_log: Vec::new(),
            remote_monster: None,
            battle_state: None,
            fighter_id: None,
            server_status: ServerStatus::Unknown,
            status_rx: None,
            show_update_modal: false,
            server_version: None,
            version_rx: None,
            tokio_rt,
            net_rx: None,
            net_task: None,
            pvp_attack_tx: None,
            monster_select_index: 0,
        })
    }

    /// Le joueur a-t-il un monstre vivant ?
    pub fn has_living_monster(&self) -> bool {
        self.storage
            .list_alive()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.check_all_aging();
        self.start_server_ping();
        self.start_version_check();

        // Start title screen music
        crate::audio::play_title_music();

        let result = self.main_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let mut battle_effects: Vec<tachyonfx::Effect> = Vec::new();
        let mut last_frame = Instant::now();
        let mut last_msg_counter: u64 = 0;
        let mut had_battle = false;
        let mut last_aging_check = Instant::now();
        // Style d'effet en attente : on le crée APRÈS le premier rendu du message.
        let mut pending_effect_style: Option<MessageStyle> = None;
        let mut pending_effect_player: Option<monster_battle_core::battle::BattleMonster> = None;
        let mut pending_effect_opponent: Option<monster_battle_core::battle::BattleMonster> = None;

        loop {
            let elapsed = last_frame.elapsed();
            last_frame = Instant::now();

            // Vérifier les événements réseau en arrière-plan
            self.poll_network();
            self.poll_server_status();
            self.poll_version_check();

            // Vérifier le vieillissement toutes les 60 secondes
            if last_aging_check.elapsed() >= Duration::from_secs(60) {
                last_aging_check = Instant::now();
                self.check_all_aging();
            }

            // Tick du combat interactif (animation HP, etc.)
            if let Some(ref mut battle) = self.battle_state {
                battle.tick();

                // Effet d'intro au début du combat
                if !had_battle {
                    battle_effects.push(fx::coalesce(EffectTimer::from_ms(
                        600,
                        Interpolation::QuadOut,
                    )));
                    had_battle = true;
                }

                // Si un effet était en attente du rendu précédent, le créer maintenant
                if let Some(style) = pending_effect_style.take() {
                    if let (Some(player), Some(opponent)) =
                        (pending_effect_player.take(), pending_effect_opponent.take())
                    {
                        Self::push_battle_effects(&mut battle_effects, &style, &player, &opponent);
                    }
                }

                // Détecter les nouveaux messages → différer l'effet au prochain frame
                if battle.message_counter != last_msg_counter {
                    last_msg_counter = battle.message_counter;
                    if let Some(ref msg) = battle.current_message {
                        pending_effect_style = Some(msg.style.clone());
                        pending_effect_player = Some(battle.player.clone());
                        pending_effect_opponent = Some(battle.opponent.clone());
                    }
                }
            } else if had_battle {
                // Combat terminé — nettoyage
                battle_effects.clear();
                had_battle = false;
                last_msg_counter = 0;
                pending_effect_style = None;
                pending_effect_player = None;
                pending_effect_opponent = None;
            }

            // Gestion du blink du curseur
            if self.blink_timer.elapsed() > Duration::from_millis(500) {
                self.name_input_blink = !self.name_input_blink;
                self.blink_timer = Instant::now();
            }

            terminal.draw(|frame| {
                screens::draw(frame, self);

                // Appliquer les effets tachyonfx par-dessus le rendu
                if !battle_effects.is_empty() {
                    let area = frame.area();
                    let buf = frame.buffer_mut();
                    battle_effects.retain_mut(|effect| {
                        effect.process(elapsed.into(), buf, area);
                        !effect.done()
                    });
                }

                // Modale de mise à jour par-dessus tout
                if self.show_update_modal {
                    crate::modales::update::draw(frame, self.server_version.as_deref());
                }
            })?;

            // Poll avec timeout pour le blink
            if crossterm::event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    self.handle_key(key.code);
                }
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        // Modale de mise à jour : seule Enter ou Esc ferme
        if self.show_update_modal {
            if matches!(code, KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')) {
                self.show_update_modal = false;
            }
            return;
        }

        // Combat interactif
        if self.current_screen == Screen::Battle {
            self.handle_battle_key(code);
            return;
        }

        // Si on est en mode saisie de nom (nouveau monstre)
        if let Screen::NamingMonster { type_index } = self.current_screen {
            match code {
                KeyCode::Char(c) => {
                    if self.name_input.len() < 20 {
                        self.name_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    self.name_input.pop();
                }
                KeyCode::Enter => {
                    if !self.name_input.trim().is_empty() {
                        self.create_starter_monster(type_index);
                    } else {
                        self.message = Some("Le nom ne peut pas être vide !".to_string());
                    }
                }
                KeyCode::Esc => {
                    self.name_input.clear();
                    self.current_screen = Screen::NewMonster;
                    self.message = None;
                }
                _ => {}
            }
            return;
        }

        // Écran d'aide — scroll & retour
        if self.current_screen == Screen::Help {
            match code {
                KeyCode::Up => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.scroll_offset += 1;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.scroll_offset = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
                _ => {}
            }
            return;
        }

        // Si on consulte le résultat d'une reproduction
        if matches!(self.current_screen, Screen::Breeding(BreedPhase::Result)) {
            match code {
                KeyCode::Up => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.scroll_offset += 1;
                }
                KeyCode::Enter | KeyCode::Char('q') | KeyCode::Esc => {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.scroll_offset = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
                _ => {}
            }
            return;
        }

        // Saisie du nom du bébé (breeding)
        if matches!(
            self.current_screen,
            Screen::Breeding(BreedPhase::NamingChild)
        ) {
            match code {
                KeyCode::Char(c) => {
                    if self.name_input.len() < 20 {
                        self.name_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    self.name_input.pop();
                }
                KeyCode::Enter => {
                    if !self.name_input.trim().is_empty() {
                        self.finalize_breeding();
                    } else {
                        self.message = Some("Le nom ne peut pas être vide !".to_string());
                    }
                }
                KeyCode::Esc => {
                    self.name_input.clear();
                    self.remote_monster = None;
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
                _ => {}
            }
            return;
        }

        // Sélection du monstre (Entraînement, Combat PvP, Reproduction)
        if let Screen::SelectMonster(ref target) = self.current_screen {
            let target = target.clone();
            let monster_count = self.storage.list_alive().map(|v| v.len()).unwrap_or(0);
            match code {
                KeyCode::Up => {
                    if self.monster_select_index > 0 {
                        self.monster_select_index -= 1;
                        crate::audio::sfx_menu_move();
                    }
                }
                KeyCode::Down => {
                    if monster_count > 0 && self.monster_select_index < monster_count - 1 {
                        self.monster_select_index += 1;
                        crate::audio::sfx_menu_move();
                    }
                }
                KeyCode::Enter => {
                    if monster_count > 0 {
                        crate::audio::sfx_menu_select();
                        match target {
                            SelectMonsterTarget::Training => {
                                self.current_screen = Screen::Training { wild: false };
                                self.menu_index = 0;
                            }
                            SelectMonsterTarget::CombatPvP => {
                                crate::audio::play_battle_music();
                                self.run_pvp();
                            }
                            SelectMonsterTarget::Breeding => {
                                crate::audio::play_breeding_music();
                                self.run_breeding();
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
                _ => {}
            }
            return;
        }

        // Erreurs PvP/Breeding
        if matches!(
            self.current_screen,
            Screen::Combat(PvpPhase::Error(_)) | Screen::Breeding(BreedPhase::Error(_))
        ) {
            match code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
                _ => {}
            }
            return;
        }

        // Écrans d'attente — Esc pour annuler
        if matches!(
            self.current_screen,
            Screen::Combat(PvpPhase::Searching)
                | Screen::Combat(PvpPhase::Matched { .. })
                | Screen::Breeding(BreedPhase::Searching)
                | Screen::Breeding(BreedPhase::Matched { .. })
        ) {
            if code == KeyCode::Esc {
                self.cancel_network();
                self.current_screen = Screen::MainMenu;
                self.menu_index = 0;
                self.message = None;
                crate::audio::play_title_music();
            }
            return;
        }

        // Écran « Mes Monstres » — navigation propre
        if self.current_screen == Screen::MonsterList {
            let monster_count = self.storage.list_alive().map(|v| v.len()).unwrap_or(0);
            match code {
                KeyCode::Char('m') | KeyCode::Char('M') => {
                    crate::audio::toggle_mute();
                }
                KeyCode::Up => {
                    if self.monster_select_index > 0 {
                        self.monster_select_index -= 1;
                        crate::audio::sfx_menu_move();
                    }
                }
                KeyCode::Down => {
                    if monster_count > 1
                        && self.monster_select_index < monster_count.saturating_sub(1)
                    {
                        self.monster_select_index += 1;
                        crate::audio::sfx_menu_move();
                    }
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    self.feed_monster();
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
                _ => {}
            }
            return;
        }

        // Navigation standard
        match code {
            KeyCode::Char('m') | KeyCode::Char('M') => {
                crate::audio::toggle_mute();
                return;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.current_screen == Screen::MainMenu {
                    self.should_quit = true;
                } else {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                    crate::audio::play_title_music();
                }
            }
            KeyCode::Up => {
                if self.menu_index > 0 {
                    self.menu_index -= 1;
                    crate::audio::sfx_menu_move();
                }
            }
            KeyCode::Down => {
                self.menu_index += 1;
                crate::audio::sfx_menu_move();
            }
            KeyCode::Left | KeyCode::Right => {
                // Toggle mode docile <-> sauvage sur l'écran d'entraînement
                if let Screen::Training { wild } = self.current_screen {
                    self.current_screen = Screen::Training { wild: !wild };
                }
            }
            KeyCode::Enter => {
                crate::audio::sfx_menu_select();
                self.handle_enter();
            }
            _ => {}
        }
    }

    fn handle_enter(&mut self) {
        match self.current_screen {
            Screen::MainMenu => {
                self.handle_main_menu_enter();
            }
            Screen::NewMonster => {
                // Passer à la saisie du nom
                use monster_battle_core::types::ElementType;
                let types = ElementType::all();
                let type_index = self.menu_index % types.len();
                self.name_input.clear();
                self.current_screen = Screen::NamingMonster { type_index };
                self.message = None;
            }
            Screen::Training { wild } => {
                crate::audio::play_battle_music();
                self.run_training_fight(wild);
            }
            _ => {}
        }
    }

    fn handle_main_menu_enter(&mut self) {
        let has_monster = self.has_living_monster();

        // Le menu est dynamique, recalculer les options
        let mut idx = 0;

        // 0 : Mes Monstres
        if self.menu_index == idx {
            self.monster_select_index = 0;
            self.current_screen = Screen::MonsterList;
            self.menu_index = 0;
            crate::audio::play_exploration_music();
            return;
        }
        idx += 1;

        // 1 : Nouveau Monstre (seulement si aucun monstre vivant)
        if !has_monster {
            if self.menu_index == idx {
                self.current_screen = Screen::NewMonster;
                self.menu_index = 0;
                return;
            }
            idx += 1;
        }

        // Entraînement (si monstre vivant) — sélection du monstre d'abord
        if has_monster {
            if self.menu_index == idx {
                self.monster_select_index = 0;
                self.current_screen = Screen::SelectMonster(SelectMonsterTarget::Training);
                self.menu_index = 0;
                return;
            }
            idx += 1;

            // Combat PvP — sélection du monstre d'abord
            if self.menu_index == idx {
                self.monster_select_index = 0;
                self.current_screen = Screen::SelectMonster(SelectMonsterTarget::CombatPvP);
                self.menu_index = 0;
                return;
            }
            idx += 1;

            // Reproduction — sélection du monstre d'abord
            if self.menu_index == idx {
                self.monster_select_index = 0;
                self.current_screen = Screen::SelectMonster(SelectMonsterTarget::Breeding);
                self.menu_index = 0;
                return;
            }
            idx += 1;
        }

        // Cimetière
        if self.menu_index == idx {
            self.current_screen = Screen::Cemetery;
            self.menu_index = 0;
            crate::audio::play_cemetery_music();
            return;
        }
        idx += 1;

        // Aide
        if self.menu_index == idx {
            self.current_screen = Screen::Help;
            self.menu_index = 0;
            self.scroll_offset = 0;
            return;
        }
        idx += 1;

        // Quitter
        if self.menu_index == idx {
            self.should_quit = true;
        }
    }

    // ==========================================
    // PvP COMBAT (non-bloquant via serveur relais)
    // ==========================================

    /// Lance une recherche de combat PvP via le serveur relais — tâche en arrière-plan.
    fn run_pvp(&mut self) {
        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let idx = self.monster_select_index.min(monsters.len() - 1);
        self.current_screen = Screen::Combat(PvpPhase::Searching);
        self.fighter_id = Some(monsters[idx].id);

        let server_addr = self.server_address.clone();
        let my_monster = monsters[idx].clone();
        let player_name = monsters[idx].name.clone();
        let (tx, rx) = mpsc::channel();

        // Canal pour envoyer les choix d'attaque au serveur pendant le combat
        let (attack_tx, mut attack_rx) = tokio::sync::mpsc::channel::<usize>(1);
        self.pvp_attack_tx = Some(attack_tx);

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<(), anyhow::Error> = async {
                let client = GameClient::new();
                client.connect(&server_addr).await?;

                // S'inscrire dans la file de combat
                client
                    .send(&NetMessage::Queue {
                        action: NetAction::Combat,
                        monster: my_monster.clone(),
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

                // Envoyer le monstre adverse → le client crée le BattleState local
                let _ = tx.send(NetworkEvent::CombatOpponentReceived(opponent_monster));

                // ── Boucle de combat PvP ──
                // Attendre les choix d'attaque du joueur local et relayer au serveur,
                // puis recevoir les résultats de tour du serveur.
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
                        // Signal PvpReady (joueur a fini de lire les messages)
                        client.send(&NetMessage::PvpReady).await?;

                        // Attendre PvpNextTurn du serveur
                        loop {
                            let msg = client.recv().await?;
                            match msg {
                                NetMessage::PvpNextTurn => {
                                    // Signaler au client qu'il peut choisir sa prochaine attaque
                                    let _ = tx.send(NetworkEvent::PvpNextTurn);
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
                        // Envoyer le choix au serveur
                        client
                            .send(&NetMessage::PvpAttackChoice { attack_index })
                            .await?;
                    }

                    // Attendre le résultat du tour du serveur
                    let msg = match client.recv().await {
                        Ok(msg) => msg,
                        Err(_) => {
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

        self.net_task = Some(handle);
        self.net_rx = Some(rx);
    }

    // ==========================================
    // REPRODUCTION (non-bloquant via serveur relais)
    // ==========================================

    /// Lance une recherche de partenaire de reproduction via le serveur relais.
    fn run_breeding(&mut self) {
        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let idx = self.monster_select_index.min(monsters.len() - 1);
        self.current_screen = Screen::Breeding(BreedPhase::Searching);
        self.fighter_id = Some(monsters[idx].id);

        let server_addr = self.server_address.clone();
        let my_monster = monsters[idx].clone();
        let player_name = monsters[idx].name.clone();
        let (tx, rx) = mpsc::channel();

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<Monster, anyhow::Error> = async {
                let client = GameClient::new();
                client.connect(&server_addr).await?;

                // S'inscrire dans la file de reproduction
                client
                    .send(&NetMessage::Queue {
                        action: NetAction::Breed,
                        monster: my_monster,
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

        self.net_task = Some(handle);
        self.net_rx = Some(rx);
    }

    fn create_starter_monster(&mut self, type_index: usize) {
        use monster_battle_core::genetics::generate_starter_stats;
        use monster_battle_core::types::ElementType;

        if self.has_living_monster() {
            self.message = Some(
                "Vous avez déjà un monstre vivant ! Obtenez-en d'autres via la reproduction."
                    .to_string(),
            );
            self.current_screen = Screen::MainMenu;
            self.menu_index = 0;
            return;
        }

        let types = ElementType::all();
        let chosen_type = types[type_index % types.len()];
        let stats = generate_starter_stats(chosen_type);
        let name = self.name_input.trim().to_string();

        let monster = Monster::new_starter(name.clone(), chosen_type, stats);

        match self.storage.save(&monster) {
            Ok(()) => {
                self.message = Some(format!("🥚 {} est né ! Prenez-en soin.", name));
            }
            Err(e) => {
                self.message = Some(format!("Erreur : {}", e));
            }
        }

        self.name_input.clear();
        self.current_screen = Screen::MonsterList;
        self.menu_index = 0;
    }

    fn run_training_fight(&mut self, wild: bool) {
        use monster_battle_core::genetics::generate_training_opponent;
        use monster_battle_core::types::ElementType;

        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let idx = self.monster_select_index.min(monsters.len() - 1);
        let fighter = &monsters[idx];

        let types = ElementType::all();
        let bot_type = types[self.menu_index % types.len()];

        let bot = generate_training_opponent(fighter.level, bot_type, wild);

        // Lancer le combat interactif
        // is_training = true (docile, 50% XP) / false (sauvage, 100% XP)
        self.fighter_id = Some(fighter.id);
        let battle = BattleState::new(fighter, &bot, !wild);
        self.battle_state = Some(battle);
        self.current_screen = Screen::Battle;
    }

    // ==========================================
    // COMBAT INTERACTIF — gestion des touches
    // ==========================================

    fn handle_battle_key(&mut self, code: KeyCode) {
        // Actions possibles calculées avec le borrow sur battle_state
        enum Action {
            None,
            End,
            Forfeit,
            PvpForfeit,
            PvpSendAttack(usize),
            PvpSendReady,
        }

        let is_pvp = self.pvp_attack_tx.is_some();

        let action = if let Some(battle) = &mut self.battle_state {
            match &battle.phase {
                BattlePhase::PlayerChooseAttack => {
                    let n = battle.player.attacks.len();
                    match code {
                        KeyCode::Up => {
                            if battle.attack_menu_index > 0 {
                                battle.attack_menu_index -= 1;
                            }
                            Action::None
                        }
                        KeyCode::Down => {
                            if battle.attack_menu_index < n - 1 {
                                battle.attack_menu_index += 1;
                            }
                            Action::None
                        }
                        KeyCode::Enter => {
                            let idx = battle.attack_menu_index;
                            if is_pvp {
                                Action::PvpSendAttack(idx)
                            } else {
                                battle.player_attack(idx);
                                Action::None
                            }
                        }
                        KeyCode::Esc if battle.is_training => Action::Forfeit,
                        KeyCode::Esc if is_pvp => Action::PvpForfeit,
                        _ => Action::None,
                    }
                }
                BattlePhase::WaitingForOpponent => {
                    // PvP : messages à lire → avancer, sinon bloquer
                    match code {
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if !battle.message_queue.is_empty() || battle.current_message.is_some()
                            {
                                if !battle.advance_message() && battle.message_queue.is_empty() {
                                    // Dernier message lu → envoyer PvpReady au serveur
                                    Action::PvpSendReady
                                } else {
                                    Action::None
                                }
                            } else {
                                // Plus rien à lire, on attend le serveur
                                Action::None
                            }
                        }
                        KeyCode::Esc if is_pvp => Action::PvpForfeit,
                        _ => Action::None,
                    }
                }
                _ => match code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // En PvP, ne pas avancer si on attend la réponse du serveur
                        if is_pvp
                            && battle.phase == BattlePhase::Executing
                            && battle.message_queue.is_empty()
                        {
                            Action::None
                        } else if !battle.advance_message() && battle.is_over() {
                            Action::End
                        } else {
                            Action::None
                        }
                    }
                    KeyCode::Esc if battle.is_training => Action::Forfeit,
                    KeyCode::Esc if is_pvp => Action::PvpForfeit,
                    _ => Action::None,
                },
            }
        } else {
            self.current_screen = Screen::MainMenu;
            return;
        };

        match action {
            Action::None => {}
            Action::End => {
                // Jouer la musique de victoire/défaite
                if let Some(ref b) = self.battle_state {
                    if b.phase == BattlePhase::Victory {
                        crate::audio::play_victory_music();
                    } else {
                        crate::audio::play_defeat_music();
                    }
                }
                // Nettoyer le canal PvP
                self.pvp_attack_tx = None;
                self.net_rx = None;
                self.net_task = None;
                self.apply_battle_results();
            }
            Action::Forfeit => {
                crate::audio::sfx_flee();
                crate::audio::play_title_music();
                self.battle_state = None;
                self.pvp_attack_tx = None;
                self.current_screen = Screen::MainMenu;
                self.menu_index = 0;
                self.message = Some("Vous avez fui le combat d'entraînement.".to_string());
            }
            Action::PvpForfeit => {
                crate::audio::sfx_flee();
                // Envoyer le forfait au serveur — le monstre NE meurt PAS
                if let Some(ref tx) = self.pvp_attack_tx {
                    // On réutilise le canal : envoyer un signal spécial
                    // Le forfait est géré en envoyant PvpForfeit au serveur
                    // via la tâche réseau. On envoie usize::MAX comme signal.
                    let _ = tx.try_send(usize::MAX);
                }
                // Passer en mode "attente" avec un message de fuite
                if let Some(ref mut battle) = self.battle_state {
                    battle.phase = BattlePhase::Executing;
                    battle.current_message = Some(monster_battle_core::battle::BattleMessage {
                        text: "🏳️ Fuite en cours...".to_string(),
                        style: MessageStyle::Info,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    });
                    battle.message_counter += 1;
                }
            }
            Action::PvpSendAttack(idx) => {
                // Envoyer le choix au serveur via le canal
                if let Some(ref tx) = self.pvp_attack_tx {
                    let _ = tx.try_send(idx);
                }
                // Passer en mode "attente" avec un message temporaire
                if let Some(ref mut battle) = self.battle_state {
                    battle.phase = BattlePhase::Executing;
                    battle.current_message = Some(monster_battle_core::battle::BattleMessage {
                        text: "⏳ En attente de l'adversaire...".to_string(),
                        style: MessageStyle::Info,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    });
                    battle.message_counter += 1;
                }
            }
            Action::PvpSendReady => {
                // Envoyer PvpReady au serveur via le canal (sentinel usize::MAX - 1)
                if let Some(ref tx) = self.pvp_attack_tx {
                    let _ = tx.try_send(usize::MAX - 1);
                }
                // Afficher un message d'attente
                if let Some(ref mut battle) = self.battle_state {
                    battle.current_message = Some(monster_battle_core::battle::BattleMessage {
                        text: "⏳ En attente de l'adversaire...".to_string(),
                        style: MessageStyle::Info,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    });
                    battle.message_counter += 1;
                }
            }
        }
    }

    /// Crée les effets visuels tachyonfx correspondant au style d'un message de combat.
    fn push_battle_effects(
        effects: &mut Vec<tachyonfx::Effect>,
        style: &MessageStyle,
        player: &monster_battle_core::battle::BattleMonster,
        opponent: &monster_battle_core::battle::BattleMonster,
    ) {
        fn element_to_color(e: monster_battle_core::types::ElementType) -> Color {
            use monster_battle_core::types::ElementType;
            match e {
                ElementType::Normal => Color::Gray,
                ElementType::Fire => Color::Red,
                ElementType::Water => Color::Blue,
                ElementType::Plant => Color::Green,
                ElementType::Electric => Color::Yellow,
                ElementType::Earth => Color::Rgb(180, 120, 60),
                ElementType::Wind => Color::Cyan,
                ElementType::Shadow => Color::Magenta,
                ElementType::Light => Color::White,
            }
        }

        match style {
            MessageStyle::PlayerAttack => {
                crate::audio::sfx_hit();
                let color = element_to_color(player.element);
                effects.push(fx::sweep_in(
                    Direction::LeftToRight,
                    10,
                    3,
                    color,
                    EffectTimer::from_ms(350, Interpolation::QuadOut),
                ));
            }
            MessageStyle::OpponentAttack => {
                crate::audio::sfx_hit();
                let color = element_to_color(opponent.element);
                effects.push(fx::sweep_in(
                    Direction::RightToLeft,
                    10,
                    3,
                    color,
                    EffectTimer::from_ms(350, Interpolation::QuadOut),
                ));
            }
            MessageStyle::SuperEffective => {
                effects.push(fx::fade_from_fg(
                    Color::Yellow,
                    EffectTimer::from_ms(500, Interpolation::QuadOut),
                ));
            }
            MessageStyle::Critical => {
                crate::audio::sfx_critical_hit();
                effects.push(fx::fade_from_fg(
                    Color::White,
                    EffectTimer::from_ms(400, Interpolation::QuadOut),
                ));
            }
            MessageStyle::Damage => {
                effects.push(fx::fade_from_fg(
                    Color::Red,
                    EffectTimer::from_ms(300, Interpolation::Linear),
                ));
            }
            MessageStyle::Heal => {
                crate::audio::sfx_heal();
                effects.push(fx::fade_from_fg(
                    Color::Green,
                    EffectTimer::from_ms(400, Interpolation::QuadOut),
                ));
            }
            MessageStyle::Victory => {
                effects.push(fx::coalesce(EffectTimer::from_ms(
                    800,
                    Interpolation::QuadOut,
                )));
            }
            MessageStyle::Defeat => {
                effects.push(fx::fade_from_fg(
                    Color::Red,
                    EffectTimer::from_ms(600, Interpolation::QuadIn),
                ));
            }
            _ => {}
        }
    }

    /// Applique les résultats d'un combat interactif terminé.
    fn apply_battle_results(&mut self) {
        let battle = match self.battle_state.take() {
            Some(b) => b,
            None => return,
        };
        let fighter_id = self.fighter_id.take();

        let mut monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.current_screen = Screen::MainMenu;
                return;
            }
        };

        // Retrouver le monstre qui a combattu par son UUID
        let fighter = if let Some(id) = fighter_id {
            monsters.iter_mut().find(|m| m.id == id)
        } else {
            monsters.first_mut()
        };

        let fighter = match fighter {
            Some(f) => f,
            None => {
                self.current_screen = Screen::MainMenu;
                return;
            }
        };

        let is_victory = battle.phase == BattlePhase::Victory;

        if is_victory {
            fighter.wins += 1;
            let old_level = fighter.level;
            fighter.gain_xp(battle.xp_gained);
            if fighter.level > old_level {
                crate::audio::sfx_level_up();
            }
            fighter.current_hp = fighter.max_hp();
        } else {
            fighter.losses += 1;
            if battle.loser_died {
                fighter.died_at = Some(chrono::Utc::now());
            } else {
                // Entraînement docile : soigner le monstre
                fighter.current_hp = fighter.max_hp();
            }
        }

        let _ = self.storage.save(fighter);

        // Retour au menu principal avec un résumé
        self.current_screen = Screen::MainMenu;
        self.menu_index = 0;
        crate::audio::play_title_music();
        if is_victory {
            self.message = Some(format!(
                "🏆 Victoire ! +{} XP{}",
                battle.xp_gained,
                if battle.is_training {
                    " (entraînement docile)"
                } else {
                    ""
                }
            ));
        } else if !battle.loser_died {
            if battle.is_training {
                self.message =
                    Some("Défaite à l'entraînement docile — pas de pénalité !".to_string());
            } else {
                self.message = Some(
                    "🏳️ Vous avez fui le combat — votre monstre est sain et sauf.".to_string(),
                );
            }
        } else {
            crate::audio::sfx_monster_death();
            self.message = Some("💀 Défaite... Votre monstre est mort.".to_string());
        }
    }

    /// Finalise la reproduction après la saisie du nom.
    fn finalize_breeding(&mut self) {
        use monster_battle_core::genetics::breed;

        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                self.current_screen = Screen::MainMenu;
                return;
            }
        };

        let remote = match self.remote_monster.take() {
            Some(m) => m,
            None => {
                self.current_screen = Screen::Breeding(BreedPhase::Error(
                    "Données du monstre partenaire manquantes.".to_string(),
                ));
                return;
            }
        };

        // Retrouver le monstre sélectionné par son UUID
        let parent = if let Some(id) = self.fighter_id {
            monsters.iter().find(|m| m.id == id).unwrap_or(&monsters[0])
        } else {
            &monsters[0]
        };

        let child_name = self.name_input.trim().to_string();
        self.name_input.clear();

        let mut log = Vec::new();
        log.push(format!(
            "🧬 Reproduction entre {} et {} !",
            parent.name, remote.name
        ));
        log.push(String::new());

        match breed(parent, &remote, child_name) {
            Ok(result) => {
                log.push(result.description.clone());
                log.push(String::new());

                let child = &result.child;
                log.push(format!("📝 Nom : {}", child.name));
                log.push(format!(
                    "🔥 Type : {} {}",
                    child.primary_type.icon(),
                    child.primary_type
                ));
                if let Some(sec) = &child.secondary_type {
                    log.push(format!("🔥 Type secondaire : {} {}", sec.icon(), sec));
                }
                log.push(format!("📊 PV : {}", child.base_stats.hp));
                log.push(format!("⚔️  Attaque : {}", child.base_stats.attack));
                log.push(format!("🛡️  Défense : {}", child.base_stats.defense));
                log.push(format!("💨 Vitesse : {}", child.base_stats.speed));
                log.push(format!("🧬 Génération : {}", child.generation));

                if !child.traits.is_empty() {
                    let traits_str: Vec<String> =
                        child.traits.iter().map(|t| format!("{}", t)).collect();
                    log.push(format!("✨ Traits : {}", traits_str.join(", ")));
                }

                if result.mutation_occurred {
                    log.push(String::new());
                    log.push("🧬 Une mutation génétique s'est produite !".to_string());
                }

                // Sauvegarder l'enfant — il remplace le monstre actuel
                // (l'ancien monstre ne meurt pas, mais on ne peut avoir qu'un seul monstre vivant)
                // On garde l'ancien monstre en vie, et on sauvegarde l'enfant en plus
                // Note : le joueur a déjà un monstre vivant, l'enfant sera un second
                // Pour la logique "un seul monstre", on laisse le joueur choisir
                // Ici on sauvegarde l'enfant — il sera visible dans la liste
                match self.storage.save(&result.child) {
                    Ok(()) => {
                        log.push(String::new());
                        log.push(format!("🎉 {} a été sauvegardé !", result.child.name));
                    }
                    Err(e) => {
                        log.push(format!("Erreur de sauvegarde : {}", e));
                    }
                }
            }
            Err(e) => {
                log.push(format!("Erreur : {}", e));
            }
        }

        self.breeding_log = log;
        self.scroll_offset = 0;
        self.current_screen = Screen::Breeding(BreedPhase::Result);
    }

    /// Vérifie si une tâche réseau en arrière-plan a terminé.
    fn poll_network(&mut self) {
        let event = match &self.net_rx {
            Some(rx) => match rx.try_recv() {
                Ok(event) => event,
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.net_rx = None;
                    self.net_task = None;
                    return;
                }
            },
            None => return,
        };

        // Ne pas nettoyer pour Queued/Matched — la tâche continue
        match event {
            NetworkEvent::Queued => {
                // Rien de spécial — on reste sur l'écran d'attente
            }
            NetworkEvent::Matched { opponent_name } => {
                crate::audio::sfx_match_found();
                // Mettre à jour l'écran pour afficher l'adversaire trouvé
                match &self.current_screen {
                    Screen::Combat(_) => {
                        self.current_screen = Screen::Combat(PvpPhase::Matched { opponent_name });
                    }
                    Screen::Breeding(_) => {
                        self.current_screen =
                            Screen::Breeding(BreedPhase::Matched { opponent_name });
                    }
                    _ => {}
                }
            }
            NetworkEvent::CombatOpponentReceived(opponent_monster) => {
                // NE PAS nettoyer la tâche réseau — elle reste active pour le combat PvP

                if let Ok(monsters) = self.storage.list_alive() {
                    // Retrouver le monstre sélectionné par son UUID
                    let fighter = self
                        .fighter_id
                        .and_then(|id| monsters.iter().find(|m| m.id == id))
                        .or(monsters.first());

                    if let Some(fighter) = fighter {
                        // Lancer le combat interactif (BattleState local pour l'affichage)
                        let battle = BattleState::new(fighter, &opponent_monster, false);
                        self.battle_state = Some(battle);
                        self.current_screen = Screen::Battle;
                    } else {
                        self.pvp_attack_tx = None;
                        self.net_rx = None;
                        self.net_task = None;
                        self.current_screen = Screen::MainMenu;
                        crate::audio::play_title_music();
                    }
                } else {
                    self.pvp_attack_tx = None;
                    self.net_rx = None;
                    self.net_task = None;
                    self.current_screen = Screen::MainMenu;
                    crate::audio::play_title_music();
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
                if let Some(ref mut battle) = self.battle_state {
                    // Mettre à jour les PV réels depuis le serveur
                    battle.player.current_hp = player_hp;
                    battle.opponent.current_hp = opponent_hp;

                    // Configurer la phase finale si le combat est terminé
                    if battle_over {
                        if victory {
                            battle.phase = BattlePhase::Victory;
                            battle.xp_gained = xp_gained;
                        } else {
                            battle.phase = BattlePhase::Defeat;
                            battle.loser_died = loser_died;
                        }
                    } else {
                        // Phase WaitingForOpponent : le joueur lit les messages,
                        // puis envoie PvpReady. Le serveur attend les deux joueurs
                        // avant d'envoyer PvpNextTurn.
                        battle.phase = BattlePhase::WaitingForOpponent;
                        battle.attack_menu_index = 0;
                    }

                    // Pousser les messages du serveur dans la file
                    battle.push_messages(messages);

                    // Afficher le premier message
                    battle.advance_message();
                }
            }
            NetworkEvent::PvpNextTurn => {
                // Les deux joueurs sont prêts → passer au choix d'attaque
                if let Some(ref mut battle) = self.battle_state {
                    battle.turn += 1;
                    battle.phase = BattlePhase::PlayerChooseAttack;
                    battle.attack_menu_index = 0;
                }
            }
            NetworkEvent::OpponentDisconnected => {
                // L'adversaire s'est déconnecté — victoire par forfait
                if let Some(ref mut battle) = self.battle_state {
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
            }
            NetworkEvent::BreedingPartnerReceived(monster) => {
                self.net_rx = None;
                self.net_task = None;

                self.remote_monster = Some(monster);
                self.name_input.clear();
                self.current_screen = Screen::Breeding(BreedPhase::NamingChild);
                self.message = None;
            }
            NetworkEvent::NetError(e) => {
                self.net_rx = None;
                self.net_task = None;
                self.pvp_attack_tx = None;
                // Si on était en combat PvP, nettoyer le battle_state
                if self.battle_state.is_some() && self.current_screen == Screen::Battle {
                    self.battle_state = None;
                }

                match &self.current_screen {
                    Screen::Combat(_) | Screen::Battle => {
                        self.current_screen = Screen::Combat(PvpPhase::Error(e));
                    }
                    Screen::Breeding(_) => {
                        self.current_screen = Screen::Breeding(BreedPhase::Error(e));
                    }
                    _ => {
                        self.message = Some(format!("Erreur réseau : {}", e));
                    }
                }
            }
        }
    }

    /// Annule la tâche réseau en cours.
    fn cancel_network(&mut self) {
        if let Some(handle) = self.net_task.take() {
            handle.abort();
        }
        self.net_rx = None;
        self.pvp_attack_tx = None;
    }

    fn check_all_aging(&mut self) {
        if let Ok(mut monsters) = self.storage.list_alive() {
            for monster in &mut monsters {
                let died_of_age = monster.check_aging();
                let died_of_hunger = if !died_of_age {
                    monster.check_hunger()
                } else {
                    false
                };

                if died_of_age {
                    self.message = Some(format!(
                        "💀 {} est mort de vieillesse à {} jours...",
                        monster.name,
                        monster.age_days()
                    ));
                    let _ = self.storage.save(monster);
                } else if died_of_hunger {
                    self.message = Some(format!(
                        "💀 {} est mort de faim ! ({} heures sans manger)",
                        monster.name,
                        monster.hours_since_fed()
                    ));
                    let _ = self.storage.save(monster);
                }
            }
        }
    }

    // ─── Nourrir le monstre ──────────────────────────────────

    fn feed_monster(&mut self) {
        let mut monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let idx = self
            .monster_select_index
            .min(monsters.len().saturating_sub(1));
        let monster = &mut monsters[idx];
        let hunger_before = monster.hunger_level();
        let hunger_after = monster.feed();

        use monster_battle_core::HungerLevel;
        let msg = match hunger_after {
            HungerLevel::Overfed => format!(
                "🤢 {} a trop mangé ! Malus de stats... (×{:.0}%)",
                monster.name,
                hunger_after.stat_multiplier() * 100.0
            ),
            HungerLevel::Satisfied => {
                if hunger_before == HungerLevel::Starving || hunger_before == HungerLevel::Hungry {
                    format!(
                        "😊 {} est rassasié ! Boost de stats ! (×{:.0}%)",
                        monster.name,
                        hunger_after.stat_multiplier() * 100.0
                    )
                } else {
                    format!(
                        "😊 {} a bien mangé ! (×{:.0}%)",
                        monster.name,
                        hunger_after.stat_multiplier() * 100.0
                    )
                }
            }
            _ => format!("🍽️ {} a été nourri.", monster.name),
        };

        let _ = self.storage.save(monster);
        self.message = Some(msg);
    }

    // ─── Health check serveur ────────────────────────────────

    /// Lance un ping périodique vers le serveur en arrière-plan.
    /// Tente une connexion WebSocket pour vérifier la joignabilité.
    fn start_server_ping(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.status_rx = Some(rx);

        let server_addr = self.server_address.clone();

        self.tokio_rt.spawn(async move {
            loop {
                let online = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    check_server_health(&server_addr),
                )
                .await
                .unwrap_or(false);

                let status = if online {
                    ServerStatus::Online
                } else {
                    ServerStatus::Offline
                };

                if tx.send(status).is_err() {
                    break; // l'App a été droppée
                }

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        });
    }

    /// Met à jour le statut serveur depuis le channel.
    fn poll_server_status(&mut self) {
        if let Some(rx) = &self.status_rx {
            while let Ok(status) = rx.try_recv() {
                self.server_status = status;
            }
        }
    }

    // ─── Version check ──────────────────────────────────────

    /// Lance la vérification de version du serveur en arrière-plan.
    fn start_version_check(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.version_rx = Some(rx);

        let server_addr = self.server_address.clone();

        self.tokio_rt.spawn(async move {
            let version = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                check_server_version(&server_addr),
            )
            .await
            .unwrap_or(None);

            let _ = tx.send(version);
        });
    }

    /// Vérifie le résultat du check de version.
    fn poll_version_check(&mut self) {
        if let Some(rx) = &self.version_rx {
            if let Ok(server_version) = rx.try_recv() {
                if let Some(ref sv) = server_version {
                    let client_version = env!("CARGO_PKG_VERSION");
                    if sv != client_version {
                        self.server_version = Some(sv.clone());
                        self.show_update_modal = true;
                    }
                }
                self.version_rx = None;
            }
        }
    }
}

/// Retourne le répertoire de données de l'application.
fn dirs_data_dir() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        std::path::PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home).join(".local").join("share")
    } else {
        std::path::PathBuf::from(".")
    }
}
