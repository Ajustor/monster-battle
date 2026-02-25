use std::collections::VecDeque;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::attack::Attack;
use crate::monster::Monster;
use crate::types::{ElementType, Trait};

// ── Structures publiques ────────────────────────────────────────────

/// Représentation d'un monstre pendant le combat interactif.
#[derive(Debug, Clone)]
pub struct BattleMonster {
    pub name: String,
    pub element: ElementType,
    pub secondary_element: Option<ElementType>,
    pub level: u32,
    pub max_hp: u32,
    pub current_hp: u32,
    /// PV affichés (animation fluide de la barre de vie).
    pub display_hp: u32,
    pub attacks: Vec<Attack>,
    pub traits: Vec<Trait>,
    pub attack_stat: u32,
    pub defense_stat: u32,
    pub speed_stat: u32,
    pub sp_attack: u32,
    pub sp_defense: u32,
}

impl BattleMonster {
    /// Crée un `BattleMonster` à partir d'un `Monster` existant.
    pub fn from_monster(monster: &Monster) -> Self {
        let attacks = Attack::attacks_for_type(monster.primary_type, monster.secondary_type);
        let max_hp = monster.max_hp();
        BattleMonster {
            name: monster.name.clone(),
            element: monster.primary_type,
            secondary_element: monster.secondary_type,
            level: monster.level,
            max_hp,
            current_hp: max_hp,
            display_hp: max_hp,
            attacks,
            traits: monster.traits.clone(),
            attack_stat: monster.effective_attack(),
            defense_stat: monster.effective_defense(),
            speed_stat: monster.effective_speed(),
            sp_attack: monster.effective_sp_attack(),
            sp_defense: monster.effective_sp_defense(),
        }
    }

    /// Pourcentage de PV restant (0.0 – 1.0).
    pub fn hp_percent(&self) -> f64 {
        if self.max_hp == 0 {
            return 0.0;
        }
        self.current_hp as f64 / self.max_hp as f64
    }
}

// ── Enums ───────────────────────────────────────────────────────────

/// Phase du combat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattlePhase {
    /// Messages d'introduction.
    Intro,
    /// Le joueur choisit une attaque.
    PlayerChooseAttack,
    /// Exécution du tour (messages affichés un par un).
    Executing,
    /// Victoire du joueur.
    Victory,
    /// Défaite du joueur.
    Defeat,
}

/// Style visuel d'un message de combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStyle {
    Normal,
    PlayerAttack,
    OpponentAttack,
    Damage,
    Critical,
    SuperEffective,
    NotEffective,
    Heal,
    Info,
    Victory,
    Defeat,
}

/// Message affiché pendant le combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleMessage {
    pub text: String,
    pub style: MessageStyle,
    /// Si défini, met à jour les PV visuels du joueur quand le message est affiché.
    pub player_hp: Option<u32>,
    /// Si défini, met à jour les PV visuels de l'adversaire quand le message est affiché.
    pub opponent_hp: Option<u32>,
    /// Animation à déclencher quand ce message est affiché.
    pub anim_type: Option<AnimationType>,
}

/// Type d'animation en cours.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationType {
    PlayerAttack,
    OpponentAttack,
    PlayerHit,
    OpponentHit,
}

// ── BattleState ─────────────────────────────────────────────────────

/// État complet du combat interactif (machine à états).
pub struct BattleState {
    pub player: BattleMonster,
    pub opponent: BattleMonster,
    pub turn: u32,
    pub phase: BattlePhase,
    /// File de messages à afficher.
    pub message_queue: VecDeque<BattleMessage>,
    /// Message actuellement affiché.
    pub current_message: Option<BattleMessage>,
    /// Index sélectionné dans le menu d'attaques.
    pub attack_menu_index: usize,
    /// S'agit-il d'un combat d'entraînement ?
    pub is_training: bool,
    /// Frame d'animation (compteur).
    pub anim_frame: u8,
    /// Type d'animation en cours.
    pub anim_type: Option<AnimationType>,
    /// PV cible visuel du joueur (les barres animent vers cette valeur).
    pub player_target_hp: u32,
    /// PV cible visuel de l'adversaire.
    pub opponent_target_hp: u32,
    /// XP gagné par le joueur (calculé en fin de combat).
    pub xp_gained: u32,
    /// Le perdant est-il mort ?
    pub loser_died: bool,
    /// Log complet du combat (texte brut).
    pub full_log: Vec<String>,
    /// Compteur de messages affichés (utilisé par la TUI pour déclencher les effets).
    pub message_counter: u64,
}

impl BattleState {
    // ── Constructeur ────────────────────────────────────────────────

    /// Crée un nouveau combat interactif.
    pub fn new(player_monster: &Monster, opponent_monster: &Monster, is_training: bool) -> Self {
        let player = BattleMonster::from_monster(player_monster);
        let opponent = BattleMonster::from_monster(opponent_monster);

        let player_hp = player.max_hp;
        let opponent_hp = opponent.max_hp;

        let mut state = BattleState {
            player,
            opponent,
            turn: 0,
            phase: BattlePhase::Intro,
            message_queue: VecDeque::new(),
            current_message: None,
            attack_menu_index: 0,
            is_training,
            anim_frame: 0,
            anim_type: None,
            player_target_hp: player_hp,
            opponent_target_hp: opponent_hp,
            xp_gained: 0,
            loser_died: false,
            full_log: Vec::new(),
            message_counter: 0,
        };

        // Messages d'introduction
        state.queue_msg("⚔️  Un combat commence !", MessageStyle::Info);
        state.queue_msg(
            &format!(
                "{} {} Nv.{} apparaît !",
                state.opponent.element.icon(),
                state.opponent.name,
                state.opponent.level
            ),
            MessageStyle::Info,
        );
        state.queue_msg(
            &format!(
                "À toi, {} ! {} Nv.{}",
                state.player.name,
                state.player.element.icon(),
                state.player.level
            ),
            MessageStyle::Info,
        );

        // Affiche le premier message
        state.advance_message();
        state
    }

    // ── API publique ────────────────────────────────────────────────

    /// Avance au message suivant. Retourne `true` si un message a été affiché.
    pub fn advance_message(&mut self) -> bool {
        // S'il reste des messages, en afficher un
        if let Some(msg) = self.message_queue.pop_front() {
            if let Some(hp) = msg.player_hp {
                self.player_target_hp = hp;
            }
            if let Some(hp) = msg.opponent_hp {
                self.opponent_target_hp = hp;
            }
            if let Some(ref anim) = msg.anim_type {
                self.anim_type = Some(anim.clone());
                self.anim_frame = 0;
            }
            self.current_message = Some(msg);
            self.message_counter += 1;
            return true;
        }

        // Plus de messages — transition de phase
        self.current_message = None;

        match self.phase {
            BattlePhase::Intro => {
                self.phase = BattlePhase::PlayerChooseAttack;
                self.turn = 1;
                self.attack_menu_index = 0;
            }
            BattlePhase::Executing => {
                if self.player.current_hp == 0 {
                    self.phase = BattlePhase::Defeat;
                    self.loser_died = !self.is_training;
                    self.queue_end_messages(false);
                    return self.show_first_queued();
                } else if self.opponent.current_hp == 0 {
                    self.xp_gained = 50 + (self.opponent.level * 5);
                    if self.is_training {
                        self.xp_gained /= 2;
                    }
                    self.phase = BattlePhase::Victory;
                    self.queue_end_messages(true);
                    return self.show_first_queued();
                } else {
                    self.turn += 1;
                    self.phase = BattlePhase::PlayerChooseAttack;
                    self.attack_menu_index = 0;
                }
            }
            BattlePhase::Victory | BattlePhase::Defeat => {
                // Terminé — `is_over()` sera `true`.
            }
            BattlePhase::PlayerChooseAttack => {}
        }

        false
    }

    /// Le joueur sélectionne une attaque par son index.
    pub fn player_attack(&mut self, attack_index: usize) {
        if self.phase != BattlePhase::PlayerChooseAttack {
            return;
        }
        if attack_index >= self.player.attacks.len() {
            return;
        }

        let mut rng = rand::thread_rng();

        // L'IA choisit une attaque
        let opp_idx = self.ai_choose_attack(&mut rng);

        // Détermine l'ordre d'attaque (vitesse)
        let player_first = if self.player.speed_stat == self.opponent.speed_stat {
            rng.gen_bool(0.5)
        } else {
            self.player.speed_stat > self.opponent.speed_stat
        };

        self.queue_msg(&format!("── Tour {} ──", self.turn), MessageStyle::Info);

        if player_first {
            self.execute_attack(true, attack_index, &mut rng);
            if self.opponent.current_hp > 0 {
                self.execute_attack(false, opp_idx, &mut rng);
            }
        } else {
            self.execute_attack(false, opp_idx, &mut rng);
            if self.player.current_hp > 0 {
                self.execute_attack(true, attack_index, &mut rng);
            }
        }

        // Effets de fin de tour
        self.apply_end_of_turn_effects();

        self.phase = BattlePhase::Executing;
        self.advance_message();
    }

    /// Met à jour les animations (appelée chaque tick du main loop, ~100 ms).
    pub fn tick(&mut self) {
        // Animation fluide des barres de PV vers les PV cible visuels
        for is_player in [true, false] {
            let (display, target, max) = if is_player {
                (
                    &mut self.player.display_hp,
                    self.player_target_hp,
                    self.player.max_hp,
                )
            } else {
                (
                    &mut self.opponent.display_hp,
                    self.opponent_target_hp,
                    self.opponent.max_hp,
                )
            };

            let step = ((max as f64 * 0.04) as u32).max(1);

            if *display > target {
                *display = display.saturating_sub(step);
                if *display < target {
                    *display = target;
                }
            } else if *display < target {
                *display = (*display + step).min(target);
            }
        }

        // Compteur d'animation
        if self.anim_type.is_some() {
            self.anim_frame += 1;
            if self.anim_frame > 3 {
                self.anim_type = None;
                self.anim_frame = 0;
            }
        }
    }

    /// Le combat est-il terminé (plus de messages, phase finale) ?
    pub fn is_over(&self) -> bool {
        matches!(self.phase, BattlePhase::Victory | BattlePhase::Defeat)
            && self.current_message.is_none()
            && self.message_queue.is_empty()
    }

    // ── Méthodes internes ───────────────────────────────────────────

    fn queue_msg(&mut self, text: &str, style: MessageStyle) {
        self.message_queue.push_back(BattleMessage {
            text: text.to_string(),
            style,
            player_hp: None,
            opponent_hp: None,
            anim_type: None,
        });
        self.full_log.push(text.to_string());
    }

    /// Ajoute un message avec un snapshot des PV (la barre se mettra à jour à l'affichage).
    fn queue_msg_with_hp(
        &mut self,
        text: &str,
        style: MessageStyle,
        player_hp: Option<u32>,
        opponent_hp: Option<u32>,
    ) {
        self.message_queue.push_back(BattleMessage {
            text: text.to_string(),
            style,
            player_hp,
            opponent_hp,
            anim_type: None,
        });
        self.full_log.push(text.to_string());
    }

    fn show_first_queued(&mut self) -> bool {
        if let Some(msg) = self.message_queue.pop_front() {
            if let Some(hp) = msg.player_hp {
                self.player_target_hp = hp;
            }
            if let Some(hp) = msg.opponent_hp {
                self.opponent_target_hp = hp;
            }
            if let Some(ref anim) = msg.anim_type {
                self.anim_type = Some(anim.clone());
                self.anim_frame = 0;
            }
            self.current_message = Some(msg);
            self.message_counter += 1;
            true
        } else {
            false
        }
    }

    fn queue_end_messages(&mut self, player_won: bool) {
        if player_won {
            self.queue_msg(
                &format!("🏆 {} a gagné le combat !", self.player.name),
                MessageStyle::Victory,
            );
            let suffix = if self.is_training {
                " (entraînement : 50%)"
            } else {
                ""
            };
            self.queue_msg(
                &format!("📖 {} XP gagné{} !", self.xp_gained, suffix),
                MessageStyle::Info,
            );
        } else {
            // Le message "💀 est K.O." est déjà ajouté par execute_attack.
            if self.is_training {
                self.queue_msg(
                    "Pas de pénalité — c'est de l'entraînement !",
                    MessageStyle::Info,
                );
            } else {
                self.queue_msg("Vous avez perdu le combat...", MessageStyle::Defeat);
            }
        }
    }

    /// IA simple : privilégie les attaques super-efficaces, sinon aléatoire.
    fn ai_choose_attack(&self, rng: &mut impl Rng) -> usize {
        let mut best: Vec<usize> = Vec::new();
        let mut best_eff = 0.0_f64;

        for (i, atk) in self.opponent.attacks.iter().enumerate() {
            let eff = atk.element.effectiveness_against(&self.player.element);
            if eff > best_eff + f64::EPSILON {
                best_eff = eff;
                best.clear();
                best.push(i);
            } else if (eff - best_eff).abs() < f64::EPSILON {
                best.push(i);
            }
        }

        if best.is_empty() {
            rng.gen_range(0..self.opponent.attacks.len())
        } else {
            best[rng.gen_range(0..best.len())]
        }
    }

    /// Exécute une attaque (phase de lecture puis phase de mutation).
    fn execute_attack(&mut self, is_player: bool, attack_index: usize, rng: &mut impl Rng) {
        // ── Phase de lecture (aucun &mut self) ──────────────────────
        let atk_name: String;
        let def_name: String;
        let attack: Attack;
        let atk_stat: f64;
        let def_stat: f64;
        let has_berserk: bool;
        let atk_hp: u32;
        let atk_max_hp: u32;
        let has_crit: bool;
        let def_has_evasion: bool;
        let def_has_thorns: bool;
        let def_element: ElementType;
        let def_secondary: Option<ElementType>;

        {
            let attacker = if is_player {
                &self.player
            } else {
                &self.opponent
            };
            let defender = if is_player {
                &self.opponent
            } else {
                &self.player
            };

            atk_name = attacker.name.clone();
            def_name = defender.name.clone();
            attack = attacker.attacks[attack_index].clone();

            let (a, d) = if attack.is_special {
                (attacker.sp_attack as f64, defender.sp_defense as f64)
            } else {
                (attacker.attack_stat as f64, defender.defense_stat as f64)
            };
            atk_stat = a;
            def_stat = d;

            has_berserk = attacker.traits.contains(&Trait::Berserk);
            atk_hp = attacker.current_hp;
            atk_max_hp = attacker.max_hp;
            has_crit = attacker.traits.contains(&Trait::CriticalStrike);
            def_has_evasion = defender.traits.contains(&Trait::Evasion);
            def_has_thorns = defender.traits.contains(&Trait::Thorns);
            def_element = defender.element;
            def_secondary = defender.secondary_element;
        }

        // Animation d'attaque associée au message "utilise"
        let atk_anim = Some(if is_player {
            AnimationType::PlayerAttack
        } else {
            AnimationType::OpponentAttack
        });

        // ── Précision ───────────────────────────────────────────────
        if rng.gen_range(0u8..100) >= attack.accuracy {
            self.queue_msg(
                &format!("{} utilise {} !", atk_name, attack.name),
                if is_player {
                    MessageStyle::PlayerAttack
                } else {
                    MessageStyle::OpponentAttack
                },
            );
            if let Some(msg) = self.message_queue.back_mut() {
                msg.anim_type = atk_anim;
            }
            self.queue_msg("L'attaque a raté !", MessageStyle::Info);
            return;
        }

        // ── Évasion ─────────────────────────────────────────────────
        if def_has_evasion && rng.gen_bool(0.12) {
            self.queue_msg(
                &format!("{} utilise {} !", atk_name, attack.name),
                if is_player {
                    MessageStyle::PlayerAttack
                } else {
                    MessageStyle::OpponentAttack
                },
            );
            if let Some(msg) = self.message_queue.back_mut() {
                msg.anim_type = atk_anim;
            }
            self.queue_msg(
                &format!("💨 {} esquive l'attaque !", def_name),
                MessageStyle::Info,
            );
            return;
        }

        // ── Berserk ─────────────────────────────────────────────────
        let berserk_active = has_berserk && atk_hp < atk_max_hp / 4;
        if berserk_active {
            self.queue_msg(
                &format!("😡 {} entre en mode Berserk !", atk_name),
                MessageStyle::Info,
            );
        }

        // ── Calcul des dégâts ───────────────────────────────────────
        let effective_atk = atk_stat * if berserk_active { 1.5 } else { 1.0 };
        let base_damage =
            ((effective_atk * 2.0) / (def_stat + effective_atk) * attack.power as f64).max(1.0);

        let mut type_mult = attack.element.effectiveness_against(&def_element);
        if let Some(sec) = def_secondary {
            type_mult *= attack.element.effectiveness_against(&sec);
        }

        let crit_chance = if has_crit { 0.20 } else { 0.08 };
        let is_critical = rng.gen_bool(crit_chance);
        let crit_mult = if is_critical { 1.5 } else { 1.0 };

        let variance = rng.gen_range(0.85..=1.15);
        let total_damage = ((base_damage * type_mult * crit_mult * variance) as u32).max(1);

        // ── Messages d'attaque ──────────────────────────────────────
        self.queue_msg(
            &format!("{} utilise {} !", atk_name, attack.name),
            if is_player {
                MessageStyle::PlayerAttack
            } else {
                MessageStyle::OpponentAttack
            },
        );
        if let Some(msg) = self.message_queue.back_mut() {
            msg.anim_type = atk_anim;
        }

        if type_mult > 1.2 {
            self.queue_msg("C'est super efficace ! 💥", MessageStyle::SuperEffective);
        } else if type_mult < 0.8 {
            self.queue_msg("Ce n'est pas très efficace...", MessageStyle::NotEffective);
        }
        if is_critical {
            self.queue_msg("Coup critique ! 💥", MessageStyle::Critical);
        }

        // ── Application des dégâts ──────────────────────────────────
        let (actual, tenacity_saved) = self.apply_damage(!is_player, total_damage, rng);

        // Animation de dégâts attachée au message → se déclenche à l'affichage
        let hit_anim = Some(if is_player {
            AnimationType::OpponentHit
        } else {
            AnimationType::PlayerHit
        });

        // Snapshot HP après dégâts → la barre se mettra à jour à l'affichage de CE message
        let (p_hp, o_hp) = (self.player.current_hp, self.opponent.current_hp);
        self.queue_msg_with_hp(
            &format!("{} subit {} dégâts !", def_name, actual),
            MessageStyle::Damage,
            if !is_player { Some(p_hp) } else { None },
            if is_player { Some(o_hp) } else { None },
        );
        if let Some(msg) = self.message_queue.back_mut() {
            msg.anim_type = hit_anim;
        }

        if tenacity_saved {
            self.queue_msg(
                &format!("💪 {} tient bon avec 1 PV !", def_name),
                MessageStyle::Info,
            );
        }

        // ── Épines ──────────────────────────────────────────────────
        let defender_alive = if is_player {
            self.opponent.current_hp > 0
        } else {
            self.player.current_hp > 0
        };

        if def_has_thorns && defender_alive {
            let thorn = (total_damage as f64 * 0.15) as u32;
            if thorn > 0 {
                self.apply_damage(is_player, thorn, rng);
                let (p_hp, o_hp) = (self.player.current_hp, self.opponent.current_hp);
                self.queue_msg_with_hp(
                    &format!("🌵 {} subit {} dégâts d'épines !", atk_name, thorn),
                    MessageStyle::Damage,
                    if is_player { Some(p_hp) } else { None },
                    if !is_player { Some(o_hp) } else { None },
                );
            }
        }

        // ── K.O. ────────────────────────────────────────────────────
        let def_hp = if is_player {
            self.opponent.current_hp
        } else {
            self.player.current_hp
        };
        if def_hp == 0 {
            self.queue_msg(&format!("💀 {} est K.O. !", def_name), MessageStyle::Defeat);
        }
    }

    /// Applique des dégâts. Retourne `(dégâts réels, tenacity_saved)`.
    fn apply_damage(&mut self, to_player: bool, damage: u32, rng: &mut impl Rng) -> (u32, bool) {
        let m = if to_player {
            &mut self.player
        } else {
            &mut self.opponent
        };
        let actual = damage.min(m.current_hp);

        if actual >= m.current_hp {
            if m.traits.contains(&Trait::Tenacity) && rng.gen_bool(0.15) {
                let dmg = m.current_hp - 1;
                m.current_hp = 1;
                return (dmg, true);
            }
            m.current_hp = 0;
            return (actual, false);
        }

        m.current_hp -= actual;
        (actual, false)
    }

    /// Régénération de fin de tour.
    fn apply_end_of_turn_effects(&mut self) {
        // Joueur
        if self.player.current_hp > 0 && self.player.traits.contains(&Trait::Regeneration) {
            let regen = ((self.player.max_hp as f64) * 0.05) as u32;
            if regen > 0 {
                let name = self.player.name.clone();
                self.player.current_hp = (self.player.current_hp + regen).min(self.player.max_hp);
                self.queue_msg_with_hp(
                    &format!("🩹 {} régénère {} PV", name, regen),
                    MessageStyle::Heal,
                    Some(self.player.current_hp),
                    None,
                );
            }
        }
        // Adversaire
        if self.opponent.current_hp > 0 && self.opponent.traits.contains(&Trait::Regeneration) {
            let regen = ((self.opponent.max_hp as f64) * 0.05) as u32;
            if regen > 0 {
                let name = self.opponent.name.clone();
                self.opponent.current_hp =
                    (self.opponent.current_hp + regen).min(self.opponent.max_hp);
                self.queue_msg_with_hp(
                    &format!("🩹 {} régénère {} PV", name, regen),
                    MessageStyle::Heal,
                    None,
                    Some(self.opponent.current_hp),
                );
            }
        }
    }

    // ── API PvP (combat serveur-arbitré) ────────────────────────────

    /// Résout un tour PvP où les deux joueurs fournissent leur choix d'attaque.
    /// Les messages sont empilés dans `message_queue` mais `advance_message()` n'est PAS appelé.
    pub fn pvp_attack(&mut self, player_attack_index: usize, opponent_attack_index: usize) {
        if self.phase != BattlePhase::PlayerChooseAttack {
            return;
        }
        if player_attack_index >= self.player.attacks.len() {
            return;
        }
        if opponent_attack_index >= self.opponent.attacks.len() {
            return;
        }

        let mut rng = rand::thread_rng();

        // Détermine l'ordre d'attaque (vitesse)
        let player_first = if self.player.speed_stat == self.opponent.speed_stat {
            rng.gen_bool(0.5)
        } else {
            self.player.speed_stat > self.opponent.speed_stat
        };

        self.queue_msg(&format!("── Tour {} ──", self.turn), MessageStyle::Info);

        if player_first {
            self.execute_attack(true, player_attack_index, &mut rng);
            if self.opponent.current_hp > 0 {
                self.execute_attack(false, opponent_attack_index, &mut rng);
            }
        } else {
            self.execute_attack(false, opponent_attack_index, &mut rng);
            if self.player.current_hp > 0 {
                self.execute_attack(true, player_attack_index, &mut rng);
            }
        }

        // Effets de fin de tour
        self.apply_end_of_turn_effects();

        // Vérifier fin de combat
        if self.player.current_hp == 0 {
            self.loser_died = true;
            // XP pour le vainqueur (l'adversaire) — doublé car kill en PvP
            self.xp_gained = (50 + (self.player.level * 5)) * 2;
            self.phase = BattlePhase::Defeat;
            self.queue_end_messages(false);
        } else if self.opponent.current_hp == 0 {
            // XP doublé car kill en PvP
            self.xp_gained = (50 + (self.opponent.level * 5)) * 2;
            self.phase = BattlePhase::Victory;
            self.queue_end_messages(true);
        } else {
            self.turn += 1;
            self.phase = BattlePhase::PlayerChooseAttack;
        }

        // NOTE : on ne call PAS advance_message() — le serveur draine les messages.
    }

    /// Draine tous les messages en attente (pour envoi réseau).
    pub fn drain_messages(&mut self) -> Vec<BattleMessage> {
        self.message_queue.drain(..).collect()
    }

    /// Pousse des messages reçus du serveur dans la file.
    pub fn push_messages(&mut self, messages: Vec<BattleMessage>) {
        for msg in messages {
            self.full_log.push(msg.text.clone());
            self.message_queue.push_back(msg);
        }
    }
}

// ── Méthodes de permutation de perspective (PvP) ────────────────────

impl MessageStyle {
    /// Inverse la perspective joueur ↔ adversaire.
    pub fn flip(&self) -> Self {
        match self {
            Self::PlayerAttack => Self::OpponentAttack,
            Self::OpponentAttack => Self::PlayerAttack,
            Self::Victory => Self::Defeat,
            Self::Defeat => Self::Victory,
            other => other.clone(),
        }
    }
}

impl AnimationType {
    /// Inverse la perspective joueur ↔ adversaire.
    pub fn flip(&self) -> Self {
        match self {
            Self::PlayerAttack => Self::OpponentAttack,
            Self::OpponentAttack => Self::PlayerAttack,
            Self::PlayerHit => Self::OpponentHit,
            Self::OpponentHit => Self::PlayerHit,
        }
    }
}

impl BattleMessage {
    /// Retourne une copie du message avec la perspective inversée (pour l'autre joueur).
    pub fn flip_perspective(&self) -> Self {
        BattleMessage {
            text: self.text.clone(),
            style: self.style.flip(),
            player_hp: self.opponent_hp,
            opponent_hp: self.player_hp,
            anim_type: self.anim_type.as_ref().map(|a| a.flip()),
        }
    }
}
