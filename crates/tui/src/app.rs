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

use monster_battle_core::Monster;
use monster_battle_core::battle::{BattlePhase, BattleState, MessageStyle};
use monster_battle_network::{GameClient, NetAction, NetMessage, check_server_health};
use monster_battle_storage::{LocalStorage, MonsterStorage};

use crate::screens;
use crate::screens::Screen;
use crate::screens::breeding::BreedPhase;
use crate::screens::pvp::PvpPhase;

/// Événements réseau reçus des tâches en arrière-plan.
enum NetworkEvent {
    /// Mis en file d'attente sur le serveur.
    Queued,
    /// Adversaire trouvé.
    Matched { opponent_name: String },
    /// Combat PvP terminé — résultat reçu du serveur.
    PvpDone {
        winner_id: String,
        loser_died: bool,
        log: Vec<String>,
        updated_monster: Monster,
    },
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

    /// Log du dernier combat d'entraînement.
    pub training_log: Vec<String>,
    /// Offset de scroll pour le log de combat.
    pub scroll_offset: usize,

    // --- Réseau ---
    /// Adresse du serveur relais (ex: "monster-battle.darthoit.eu").
    pub server_address: String,
    /// Log du dernier combat PvP.
    pub pvp_log: Vec<String>,

    // --- Reproduction ---
    /// Log du résultat de reproduction.
    pub breeding_log: Vec<String>,
    /// Monstre du partenaire distant (reçu via réseau).
    pub remote_monster: Option<Monster>,

    /// État du combat interactif en cours (style Pokémon).
    pub battle_state: Option<BattleState>,

    /// Statut de connexion au serveur relais.
    pub server_status: ServerStatus,
    /// Récepteur pour les mises à jour du statut serveur.
    status_rx: Option<mpsc::Receiver<ServerStatus>>,

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
            server_address: std::env::var("MONSTER_SERVER")
                .unwrap_or_else(|_| "monster-battle.darthoit.eu".to_string()),
            pvp_log: Vec::new(),
            breeding_log: Vec::new(),
            remote_monster: None,
            battle_state: None,
            server_status: ServerStatus::Unknown,
            status_rx: None,
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
        self.start_server_ping();

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

            // Combat PvP — connexion directe au serveur relais
            if self.menu_index == idx {
                self.run_pvp();
                return;
            }
            idx += 1;

            // Reproduction — connexion directe au serveur relais
            if self.menu_index == idx {
                self.run_breeding();
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

        self.current_screen = Screen::Combat(PvpPhase::Searching);

        let server_addr = self.server_address.clone();
        let my_monster = monsters[0].clone();
        let player_name = monsters[0].name.clone();
        let (tx, rx) = mpsc::channel();

        let handle = self.tokio_rt.spawn(async move {
            let result: Result<_, anyhow::Error> = async {
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

                // Attendre le résultat du combat
                let msg = client.recv().await?;
                match msg {
                    NetMessage::CombatResult {
                        winner_id,
                        loser_died,
                        log,
                        updated_monster,
                        ..
                    } => {
                        return Ok((winner_id, loser_died, log, updated_monster));
                    }
                    NetMessage::Error(e) => return Err(anyhow::anyhow!("{}", e)),
                    _ => return Err(anyhow::anyhow!("Résultat inattendu.")),
                }
            }
            .await;

            let event = match result {
                Ok((winner_id, loser_died, log, updated_monster)) => NetworkEvent::PvpDone {
                    winner_id,
                    loser_died,
                    log,
                    updated_monster,
                },
                Err(e) => NetworkEvent::NetError(format!("{}", e)),
            };
            let _ = tx.send(event);
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

        self.current_screen = Screen::Breeding(BreedPhase::Searching);

        let server_addr = self.server_address.clone();
        let my_monster = monsters[0].clone();
        let player_name = monsters[0].name.clone();
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

        // Ne pas nettoyer pour Queued/Matched — la tâche continue
        match event {
            NetworkEvent::Queued => {
                // Rien de spécial — on reste sur l'écran d'attente
            }
            NetworkEvent::Matched { opponent_name } => {
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
            NetworkEvent::PvpDone {
                winner_id,
                loser_died,
                log,
                updated_monster,
            } => {
                // Nettoyer la tâche réseau
                self.net_rx = None;
                self.net_task = None;

                if let Ok(mut monsters) = self.storage.list_alive() {
                    if !monsters.is_empty() {
                        let mut pvp_log = Vec::new();
                        pvp_log.push(format!("⚔️  Combat PvP : {} !", monsters[0].name));
                        pvp_log.push(String::new());
                        pvp_log.extend(log);
                        pvp_log.push(String::new());

                        let my_id = monsters[0].id.to_string();
                        if winner_id == my_id {
                            pvp_log.push(format!("🏆 {} a gagné !", monsters[0].name));
                            monsters[0].wins += 1;
                            // XP basée sur le niveau du monstre adverse
                            let xp = 50 + (updated_monster.level * 5);
                            monsters[0].gain_xp(xp);
                            monsters[0].current_hp = monsters[0].max_hp();
                        } else {
                            pvp_log.push(format!("💀 {} a perdu...", monsters[0].name));
                            monsters[0].losses += 1;
                            if loser_died {
                                monsters[0].died_at = Some(chrono::Utc::now());
                                pvp_log
                                    .push(format!("☠️  {} est mort au combat !", monsters[0].name));
                            }
                        }

                        let _ = self.storage.save(&monsters[0]);
                        self.pvp_log = pvp_log;
                    }
                }
                self.scroll_offset = 0;
                self.current_screen = Screen::Combat(PvpPhase::Result);
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

                match &self.current_screen {
                    Screen::Combat(_) => {
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
