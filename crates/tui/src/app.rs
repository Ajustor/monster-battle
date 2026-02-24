use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use monster_battle_core::Monster;
use monster_battle_core::battle::{BattlePhase, BattleState};
use monster_battle_network::{GameClient, GameServer, NetAction, NetMessage};
use monster_battle_storage::{LocalStorage, MonsterStorage};

use crate::screens;
use crate::screens::Screen;
use crate::screens::breeding::BreedPhase;
use crate::screens::pvp::PvpPhase;

/// Événements réseau reçus des tâches en arrière-plan.
enum NetworkEvent {
    /// Combat PvP terminé (côté hôte).
    PvpHostDone {
        updated_monster: Monster,
        pvp_log: Vec<String>,
    },
    /// Combat PvP terminé (côté client).
    PvpClientDone {
        winner_id: String,
        loser_died: bool,
        remote_log: Vec<String>,
        remote_monster_name: String,
        remote_monster_level: u32,
    },
    /// Monstre du partenaire reçu (reproduction).
    BreedingPartnerReceived(Monster),
    /// Erreur réseau.
    NetError(String),
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

    /// Log du dernier combat d'entraînement.
    pub training_log: Vec<String>,
    /// Offset de scroll pour le log de combat.
    pub scroll_offset: usize,

    // --- PvP ---
    /// Port d'écoute pour le mode hôte.
    pub pvp_port: u16,
    /// Adresse IP saisie pour le mode client.
    pub pvp_address_input: String,
    /// Log du dernier combat PvP.
    pub pvp_log: Vec<String>,

    // --- Reproduction ---
    /// Log du résultat de reproduction.
    pub breeding_log: Vec<String>,
    /// Monstre du partenaire distant (reçu via réseau).
    pub remote_monster: Option<Monster>,

    /// État du combat interactif en cours (style Pokémon).
    pub battle_state: Option<BattleState>,

    /// Adresses IP locales détectées.
    pub local_ips: Vec<String>,

    /// Runtime tokio pour les opérations réseau.
    tokio_rt: tokio::runtime::Runtime,
    /// Récepteur pour les événements réseau en arrière-plan.
    net_rx: Option<mpsc::Receiver<NetworkEvent>>,
    /// Handle de la tâche réseau en cours.
    net_task: Option<tokio::task::JoinHandle<()>>,
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
            training_log: Vec::new(),
            scroll_offset: 0,
            pvp_port: 7878,
            pvp_address_input: String::new(),
            pvp_log: Vec::new(),
            breeding_log: Vec::new(),
            remote_monster: None,
            battle_state: None,
            local_ips: detect_local_ips(),
            tokio_rt,
            net_rx: None,
            net_task: None,
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

        let result = self.main_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Vérifier les événements réseau en arrière-plan
            self.poll_network();

            // Tick du combat interactif (animation HP, etc.)
            if let Some(ref mut battle) = self.battle_state {
                battle.tick();
            }

            // Gestion du blink du curseur
            if self.blink_timer.elapsed() > Duration::from_millis(500) {
                self.name_input_blink = !self.name_input_blink;
                self.blink_timer = Instant::now();
            }

            terminal.draw(|frame| screens::draw(frame, self))?;

            // Poll avec timeout pour le blink
            if crossterm::event::poll(Duration::from_millis(100))? {
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

        // Si on consulte le résultat d'un entraînement / PvP / breeding
        if matches!(
            self.current_screen,
            Screen::TrainingResult | Screen::CombatResult | Screen::BreedingResult
        ) || matches!(self.current_screen, Screen::Combat(PvpPhase::Result))
            || matches!(self.current_screen, Screen::Breeding(BreedPhase::Result))
        {
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
                }
                _ => {}
            }
            return;
        }

        // Saisie d'adresse PvP
        if matches!(self.current_screen, Screen::Combat(PvpPhase::EnterAddress)) {
            match code {
                KeyCode::Char(c) => {
                    self.pvp_address_input.push(c);
                }
                KeyCode::Backspace => {
                    self.pvp_address_input.pop();
                }
                KeyCode::Enter => {
                    if !self.pvp_address_input.trim().is_empty() {
                        let addr = self.pvp_address_input.trim().to_string();
                        self.current_screen = Screen::Combat(PvpPhase::Connecting);
                        self.run_pvp_as_client(&addr);
                    }
                }
                KeyCode::Esc => {
                    self.pvp_address_input.clear();
                    self.current_screen = Screen::Combat(PvpPhase::Menu);
                    self.message = None;
                }
                _ => {}
            }
            return;
        }

        // Saisie d'adresse breeding
        if matches!(
            self.current_screen,
            Screen::Breeding(BreedPhase::EnterAddress)
        ) {
            match code {
                KeyCode::Char(c) => {
                    self.pvp_address_input.push(c);
                }
                KeyCode::Backspace => {
                    self.pvp_address_input.pop();
                }
                KeyCode::Enter => {
                    if !self.pvp_address_input.trim().is_empty() {
                        let addr = self.pvp_address_input.trim().to_string();
                        self.current_screen = Screen::Breeding(BreedPhase::Connecting);
                        self.run_breeding_as_client(&addr);
                    }
                }
                KeyCode::Esc => {
                    self.pvp_address_input.clear();
                    self.current_screen = Screen::Breeding(BreedPhase::Menu);
                    self.message = None;
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
                }
                _ => {}
            }
            return;
        }

        // Challenge reçu (PvP)
        if let Screen::Combat(PvpPhase::ReceivedChallenge { .. }) = &self.current_screen {
            match code {
                KeyCode::Enter => {
                    // Accepter — pas vraiment implémenté ici car on fait tout en blocking
                    self.message = Some("Défi accepté !".to_string());
                }
                KeyCode::Esc => {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                }
                _ => {}
            }
            return;
        }

        // Proposition de reproduction reçue
        if let Screen::Breeding(BreedPhase::ReceivedProposal { .. }) = &self.current_screen {
            match code {
                KeyCode::Enter => {
                    self.message = Some("Proposition acceptée !".to_string());
                }
                KeyCode::Esc => {
                    self.remote_monster = None;
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
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
                }
                _ => {}
            }
            return;
        }

        // Écrans d'attente — Esc pour annuler
        if matches!(
            self.current_screen,
            Screen::Combat(PvpPhase::WaitingForOpponent)
                | Screen::Combat(PvpPhase::WaitingForAccept)
                | Screen::Combat(PvpPhase::Connecting)
                | Screen::Combat(PvpPhase::Fighting)
                | Screen::Breeding(BreedPhase::WaitingForPartner)
                | Screen::Breeding(BreedPhase::WaitingForAccept)
                | Screen::Breeding(BreedPhase::Connecting)
        ) {
            if code == KeyCode::Esc {
                self.cancel_network();
                self.current_screen = Screen::MainMenu;
                self.menu_index = 0;
                self.message = None;
            }
            return;
        }

        // Navigation standard
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.current_screen == Screen::MainMenu {
                    self.should_quit = true;
                } else {
                    self.current_screen = Screen::MainMenu;
                    self.menu_index = 0;
                    self.message = None;
                }
            }
            KeyCode::Up => {
                if self.menu_index > 0 {
                    self.menu_index -= 1;
                }
            }
            KeyCode::Down => {
                self.menu_index += 1;
            }
            KeyCode::Enter => {
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
            Screen::Training => {
                self.run_training_fight();
            }
            Screen::Combat(PvpPhase::Menu) => {
                self.handle_pvp_menu_enter();
            }
            Screen::Breeding(BreedPhase::Menu) => {
                self.handle_breeding_menu_enter();
            }
            _ => {}
        }
    }

    fn handle_main_menu_enter(&mut self) {
        let has_monster = self.has_living_monster();

        // Le menu est dynamique, recalculer les options
        let mut idx = 0;

        // 0 : Mon Monstre
        if self.menu_index == idx {
            self.current_screen = Screen::MonsterList;
            self.menu_index = 0;
            return;
        }
        idx += 1;

        // 1 (si pas de monstre) : Nouveau Monstre
        if !has_monster {
            if self.menu_index == idx {
                self.current_screen = Screen::NewMonster;
                self.menu_index = 0;
                return;
            }
            idx += 1;
        }

        // Entraînement (si monstre vivant)
        if has_monster {
            if self.menu_index == idx {
                self.current_screen = Screen::Training;
                self.menu_index = 0;
                return;
            }
            idx += 1;

            // Combat PvP
            if self.menu_index == idx {
                self.current_screen = Screen::Combat(PvpPhase::Menu);
                self.menu_index = 0;
                return;
            }
            idx += 1;

            // Reproduction
            if self.menu_index == idx {
                self.current_screen = Screen::Breeding(BreedPhase::Menu);
                self.menu_index = 0;
                return;
            }
            idx += 1;
        }

        // Cimetière
        if self.menu_index == idx {
            self.current_screen = Screen::Cemetery;
            self.menu_index = 0;
            return;
        }
        idx += 1;

        // Quitter
        if self.menu_index == idx {
            self.should_quit = true;
        }
    }

    fn handle_pvp_menu_enter(&mut self) {
        match self.menu_index % 2 {
            0 => {
                // Héberger
                self.run_pvp_as_host();
            }
            1 => {
                // Rejoindre
                self.pvp_address_input.clear();
                self.current_screen = Screen::Combat(PvpPhase::EnterAddress);
                self.message = None;
            }
            _ => {}
        }
    }

    fn handle_breeding_menu_enter(&mut self) {
        match self.menu_index % 2 {
            0 => {
                // Héberger
                self.run_breeding_as_host();
            }
            1 => {
                // Rejoindre
                self.pvp_address_input.clear();
                self.current_screen = Screen::Breeding(BreedPhase::EnterAddress);
                self.message = None;
            }
            _ => {}
        }
    }

    fn create_starter_monster(&mut self, type_index: usize) {
        use monster_battle_core::genetics::generate_starter_stats;
        use monster_battle_core::types::ElementType;

        // Limite : un seul monstre vivant
        if self.has_living_monster() {
            self.message = Some("Vous avez déjà un monstre vivant !".to_string());
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

    fn run_training_fight(&mut self) {
        use monster_battle_core::genetics::generate_starter_stats;
        use monster_battle_core::types::ElementType;

        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let types = ElementType::all();
        let bot_type = types[self.menu_index % types.len()];

        // Créer un bot du même niveau que le joueur (±2)
        let player_level = monsters[0].level;
        let bot_level = player_level.saturating_sub(2).max(1);
        let mut bot_stats = generate_starter_stats(bot_type);
        bot_stats.hp += bot_level * 2;

        let mut bot = Monster::new_starter(format!("Bot {}", bot_type), bot_type, bot_stats);
        if bot_level > 1 {
            bot.gain_xp(bot_level * bot_level * 10);
        }

        // Lancer le combat interactif
        let battle = BattleState::new(&monsters[0], &bot, true);
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
        }

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
                            battle.player_attack(idx);
                            Action::None
                        }
                        KeyCode::Esc if battle.is_training => Action::Forfeit,
                        _ => Action::None,
                    }
                }
                _ => match code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if !battle.advance_message() && battle.is_over() {
                            Action::End
                        } else {
                            Action::None
                        }
                    }
                    KeyCode::Esc if battle.is_training => Action::Forfeit,
                    _ => Action::None,
                },
            }
        } else {
            self.current_screen = Screen::MainMenu;
            return;
        };

        match action {
            Action::None => {}
            Action::End => self.apply_battle_results(),
            Action::Forfeit => {
                self.battle_state = None;
                self.current_screen = Screen::MainMenu;
                self.message = Some("Vous avez fui le combat d'entraînement.".to_string());
            }
        }
    }

    /// Applique les résultats d'un combat interactif terminé.
    fn apply_battle_results(&mut self) {
        let battle = match self.battle_state.take() {
            Some(b) => b,
            None => return,
        };

        let mut monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.current_screen = Screen::MainMenu;
                return;
            }
        };

        let is_victory = battle.phase == BattlePhase::Victory;

        if is_victory {
            monsters[0].wins += 1;
            monsters[0].gain_xp(battle.xp_gained);
            monsters[0].current_hp = monsters[0].max_hp();
        } else {
            monsters[0].losses += 1;
            if battle.is_training {
                // Pas de mort en entraînement
                monsters[0].died_at = None;
                monsters[0].current_hp = monsters[0].max_hp();
            } else if battle.loser_died {
                monsters[0].died_at = Some(chrono::Utc::now());
            }
        }

        let _ = self.storage.save(&monsters[0]);

        // Afficher le log complet dans l'écran de résultat
        self.training_log = battle.full_log;
        self.scroll_offset = 0;
        self.current_screen = Screen::TrainingResult;
    }

    // ==========================================
    // PvP COMBAT (non-bloquant)
    // ==========================================

    /// Héberge un combat PvP (mode hôte) — tâche en arrière-plan.
    fn run_pvp_as_host(&mut self) {
        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        self.current_screen = Screen::Combat(PvpPhase::WaitingForOpponent);

        let port = self.pvp_port;
        let my_monster = monsters[0].clone();
        let (tx, rx) = mpsc::channel();

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<(Monster, Vec<String>), anyhow::Error> = async {
                let server = GameServer::new(port);
                server.accept_one().await?;

                server.send(&NetMessage::Propose(NetAction::Combat)).await?;
                server
                    .send(&NetMessage::MonsterData(my_monster.clone()))
                    .await?;

                let response = server.recv().await?;
                match response {
                    NetMessage::Accept => {}
                    NetMessage::Decline => {
                        return Err(anyhow::anyhow!("L'adversaire a refusé le combat."));
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Réponse inattendue."));
                    }
                }

                let remote_msg = server.recv().await?;
                let mut remote_monster = match remote_msg {
                    NetMessage::MonsterData(m) => m,
                    _ => return Err(anyhow::anyhow!("Données du monstre adverses manquantes.")),
                };

                // Combat
                let mut local_monster = my_monster;
                local_monster.current_hp = local_monster.max_hp();

                let fight_result =
                    monster_battle_core::combat::fight(&mut local_monster, &mut remote_monster)
                        .map_err(|e| anyhow::anyhow!(e))?;

                // Construire le log
                let mut log = Vec::new();
                log.push(format!(
                    "⚔️  Combat PvP : {} vs {} !",
                    local_monster.name, remote_monster.name
                ));
                log.push(String::new());
                for event in &fight_result.log {
                    log.push(event.describe());
                }
                log.push(String::new());

                let i_won = fight_result.winner_id == local_monster.id;
                if i_won {
                    log.push(format!("🏆 {} a gagné !", local_monster.name));
                } else {
                    log.push(format!("💀 {} a perdu...", local_monster.name));
                    if fight_result.loser_died {
                        log.push(format!("☠️  {} est mort au combat !", local_monster.name));
                    }
                }

                // Envoyer le résultat au client distant
                let combat_result_msg = NetMessage::CombatResult {
                    winner_id: fight_result.winner_id.to_string(),
                    loser_id: fight_result.loser_id.to_string(),
                    loser_died: fight_result.loser_died,
                    log: fight_result.log.iter().map(|e| e.describe()).collect(),
                    updated_monster: remote_monster,
                };
                server.send(&combat_result_msg).await?;

                Ok((local_monster, log))
            }
            .await;

            let event = match result {
                Ok((monster, log)) => NetworkEvent::PvpHostDone {
                    updated_monster: monster,
                    pvp_log: log,
                },
                Err(e) => NetworkEvent::NetError(format!("{}", e)),
            };
            let _ = tx.send(event);
        });

        self.net_task = Some(handle);
        self.net_rx = Some(rx);
    }

    /// Rejoint un combat PvP (mode client) — tâche en arrière-plan.
    fn run_pvp_as_client(&mut self, addr: &str) {
        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let my_monster = monsters[0].clone();
        let addr = addr.to_string();
        let (tx, rx) = mpsc::channel();

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<_, anyhow::Error> = async {
                let client = GameClient::new();
                client.connect(&addr).await?;

                let msg = client.recv().await?;
                match msg {
                    NetMessage::Propose(NetAction::Combat) => {}
                    _ => return Err(anyhow::anyhow!("Proposition inattendue.")),
                }

                let remote_msg = client.recv().await?;
                let remote_monster = match remote_msg {
                    NetMessage::MonsterData(m) => m,
                    _ => return Err(anyhow::anyhow!("Données manquantes.")),
                };

                client.send(&NetMessage::Accept).await?;
                client.send(&NetMessage::MonsterData(my_monster)).await?;

                let result_msg = client.recv().await?;
                match result_msg {
                    NetMessage::CombatResult {
                        winner_id,
                        loser_died,
                        log,
                        updated_monster,
                        ..
                    } => Ok((
                        winner_id,
                        loser_died,
                        log,
                        updated_monster.name.clone(),
                        updated_monster.level,
                    )),
                    NetMessage::Error(e) => Err(anyhow::anyhow!("Erreur : {}", e)),
                    _ => Err(anyhow::anyhow!("Résultat inattendu.")),
                }
            }
            .await;

            let event = match result {
                Ok((winner_id, loser_died, remote_log, name, level)) => {
                    NetworkEvent::PvpClientDone {
                        winner_id,
                        loser_died,
                        remote_log,
                        remote_monster_name: name,
                        remote_monster_level: level,
                    }
                }
                Err(e) => NetworkEvent::NetError(format!("{}", e)),
            };
            let _ = tx.send(event);
        });

        self.net_task = Some(handle);
        self.net_rx = Some(rx);
    }

    // ==========================================
    // REPRODUCTION (non-bloquant)
    // ==========================================

    /// Héberge une session de reproduction (mode hôte) — tâche en arrière-plan.
    fn run_breeding_as_host(&mut self) {
        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        self.current_screen = Screen::Breeding(BreedPhase::WaitingForPartner);

        let port = self.pvp_port;
        let my_monster = monsters[0].clone();
        let (tx, rx) = mpsc::channel();

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<Monster, anyhow::Error> = async {
                let server = GameServer::new(port);
                server.accept_one().await?;

                server.send(&NetMessage::Propose(NetAction::Breed)).await?;
                server.send(&NetMessage::MonsterData(my_monster)).await?;

                let response = server.recv().await?;
                match response {
                    NetMessage::Accept => {}
                    NetMessage::Decline => {
                        return Err(anyhow::anyhow!("L'autre joueur a refusé la reproduction."));
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Réponse inattendue."));
                    }
                }

                let remote_msg = server.recv().await?;
                let remote_monster = match remote_msg {
                    NetMessage::MonsterData(m) => m,
                    _ => return Err(anyhow::anyhow!("Données du monstre manquantes.")),
                };

                Ok(remote_monster)
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

    /// Rejoint une session de reproduction (mode client) — tâche en arrière-plan.
    fn run_breeding_as_client(&mut self, addr: &str) {
        let monsters = match self.storage.list_alive() {
            Ok(m) if !m.is_empty() => m,
            _ => {
                self.message = Some("Pas de monstre vivant !".to_string());
                return;
            }
        };

        let my_monster = monsters[0].clone();
        let addr = addr.to_string();
        let (tx, rx) = mpsc::channel();

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<Monster, anyhow::Error> = async {
                let client = GameClient::new();
                client.connect(&addr).await?;

                let msg = client.recv().await?;
                match msg {
                    NetMessage::Propose(NetAction::Breed) => {}
                    _ => return Err(anyhow::anyhow!("Proposition inattendue.")),
                }

                let remote_msg = client.recv().await?;
                let remote_monster = match remote_msg {
                    NetMessage::MonsterData(m) => m,
                    _ => return Err(anyhow::anyhow!("Données manquantes.")),
                };

                client.send(&NetMessage::Accept).await?;
                client.send(&NetMessage::MonsterData(my_monster)).await?;

                Ok(remote_monster)
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

        let child_name = self.name_input.trim().to_string();
        self.name_input.clear();

        let mut log = Vec::new();
        log.push(format!(
            "🧬 Reproduction entre {} et {} !",
            monsters[0].name, remote.name
        ));
        log.push(String::new());

        match breed(&monsters[0], &remote, child_name) {
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

        // Nettoyer
        self.net_rx = None;
        self.net_task = None;

        match event {
            NetworkEvent::PvpHostDone {
                updated_monster,
                pvp_log,
            } => {
                let _ = self.storage.save(&updated_monster);
                self.pvp_log = pvp_log;
                self.scroll_offset = 0;
                self.current_screen = Screen::Combat(PvpPhase::Result);
            }
            NetworkEvent::PvpClientDone {
                winner_id,
                loser_died,
                remote_log,
                remote_monster_name,
                remote_monster_level,
            } => {
                if let Ok(mut monsters) = self.storage.list_alive() {
                    if !monsters.is_empty() {
                        let mut log = Vec::new();
                        log.push(format!(
                            "⚔️  Combat PvP : {} vs {} !",
                            monsters[0].name, remote_monster_name
                        ));
                        log.push(String::new());
                        log.extend(remote_log);
                        log.push(String::new());

                        let my_id = monsters[0].id.to_string();
                        if winner_id == my_id {
                            log.push(format!("🏆 {} a gagné !", monsters[0].name));
                            monsters[0].wins += 1;
                            let xp = 50 + (remote_monster_level * 5);
                            monsters[0].gain_xp(xp);
                        } else {
                            log.push(format!("💀 {} a perdu...", monsters[0].name));
                            monsters[0].losses += 1;
                            if loser_died {
                                monsters[0].died_at = Some(chrono::Utc::now());
                                log.push(format!("☠️  {} est mort au combat !", monsters[0].name));
                            }
                        }

                        let _ = self.storage.save(&monsters[0]);
                        self.pvp_log = log;
                    }
                }
                self.scroll_offset = 0;
                self.current_screen = Screen::Combat(PvpPhase::Result);
            }
            NetworkEvent::BreedingPartnerReceived(monster) => {
                self.remote_monster = Some(monster);
                self.name_input.clear();
                self.current_screen = Screen::Breeding(BreedPhase::NamingChild);
                self.message = None;
            }
            NetworkEvent::NetError(e) => match &self.current_screen {
                Screen::Combat(_) => {
                    self.current_screen = Screen::Combat(PvpPhase::Error(e));
                }
                Screen::Breeding(_) => {
                    self.current_screen = Screen::Breeding(BreedPhase::Error(e));
                }
                _ => {
                    self.message = Some(format!("Erreur réseau : {}", e));
                }
            },
        }
    }

    /// Annule la tâche réseau en cours.
    fn cancel_network(&mut self) {
        if let Some(handle) = self.net_task.take() {
            handle.abort();
        }
        self.net_rx = None;
    }

    fn check_all_aging(&mut self) {
        if let Ok(mut monsters) = self.storage.list_alive() {
            for monster in &mut monsters {
                if monster.check_aging() {
                    self.message = Some(format!(
                        "💀 {} est mort de vieillesse à {} jours...",
                        monster.name,
                        monster.age_days()
                    ));
                    let _ = self.storage.save(monster);
                }
            }
        }
    }
}

/// Détecte les adresses IP locales de la machine.
fn detect_local_ips() -> Vec<String> {
    let mut ips = Vec::new();

    // Lire les interfaces réseau via /proc/net (Linux)
    if let Ok(output) = std::process::Command::new("hostname").arg("-I").output() {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for ip in stdout.split_whitespace() {
                    // Filtrer les adresses IPv4 non-loopback
                    if ip.contains('.') && !ip.starts_with("127.") {
                        ips.push(ip.to_string());
                    }
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    ips
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
