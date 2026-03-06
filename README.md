# 🐉 Monster Battle

Un jeu de monstres en terminal (TUI) inspiré de Pokémon et des Tamagotchi, écrit en Rust.

Adopte un monstre, entraîne-le, fais-le combattre en ligne contre d'autres joueurs, et reproduis-le pour créer des lignées toujours plus puissantes — mais attention, **tes monstres vieillissent et finissent par mourir**.

## ✨ Fonctionnalités

- **8 types élémentaires** : Feu 🔥, Eau 💧, Plante 🌿, Électrique ⚡, Terre 🪨, Vent 🌪️, Ombre 🌑, Lumière ✨ (+ Normal ⭐ pour les attaques)
- **Système de combat interactif** style Pokémon avec animations en terminal (tachyonfx)
- **Entraînement** contre des bots pour gagner de l'XP (50 % de l'XP d'un vrai combat)
- **Combat PvP en ligne** via un serveur relais centralisé — matchmaking automatique
- **Reproduction** entre monstres de joueurs différents pour créer des hybrides avec types secondaires, héritage de stats et mutations de traits génétiques
- **Vieillissement et mort** : chaque monstre a une durée de vie limitée (~30 jours, +15 avec le trait Longévité)
- **8 traits génétiques** : Régénération, Évasion, Coup Critique+, Longévité, Apprentissage Rapide, Épines, Berserk, Ténacité
- **Sauvegarde locale chiffrée** (AES-256-GCM) liée à la machine
- **Cimetière** pour honorer les monstres disparus

## 🏗️ Architecture

Le projet est un workspace Cargo composé de 5 crates :

```
monster-battle/
├── crates/
│   ├── core/       # Logique métier : monstres, types, stats, combat, génétique
│   ├── storage/    # Sauvegarde locale chiffrée (AES-256-GCM)
│   ├── network/    # Client TCP + protocole réseau (sérialisation JSON)
│   ├── tui/        # Interface terminal (ratatui + crossterm + tachyonfx)
│   └── server/     # Serveur relais centralisé (matchmaking, combat, reproduction)
└── Cargo.toml      # Workspace root
```

### `core`
Moteur du jeu, sans aucune dépendance I/O. Contient :
- `Monster` — structure complète d'un monstre (stats, types, traits, XP, âge, lignée…)
- `ElementType` — 9 éléments avec table d'efficacité (×1.5 super efficace, ×0.5 résisté)
- `Stats` — HP, Attaque, Défense, Vitesse, Attaque Spé, Défense Spé
- `Trait` — 8 traits génétiques héritables/mutables
- `Attack` — attaques physiques et spéciales liées aux types
- `genetics::breed()` — reproduction avec héritage de stats et mutations
- `BattleState` — combat interactif style Pokémon

### `storage`
Persistance locale avec chiffrement :
- Les monstres vivants sont chiffrés en AES-256-GCM (fichiers `.enc`)
- Les monstres morts sont stockés en clair (fichiers `.json`) dans le cimetière
- La clé de chiffrement est dérivée d'un identifiant machine (non portable volontairement)

### `network`
Couche réseau client + protocole :
- `GameClient` — client TCP asynchrone (tokio)
- `NetMessage` — protocole JSON sérialisé avec préfixe de longueur (`Queue`, `Matched`, `CombatOpponent`, `PvpAttackChoice`, `PvpTurnResult`, `BreedingPartner`, `Ping`/`Pong`, etc.)
- `NetAction` — type d'action : `Combat` ou `Breed`

### `tui`
Interface terminal interactive :
- Rendu avec **ratatui** + **crossterm**
- Animations de combat avec **tachyonfx**
- Écrans : menu principal, liste des monstres, création, entraînement, combat PvP, reproduction, cimetière
- Indicateur de connexion serveur en temps réel (🟢 / 🔴)

### `server`
Serveur relais centralisé (binaire séparé) :
- TCP port **7878** — protocole de jeu (matchmaking, combat, reproduction) + route santé HTTP
- Détection automatique HTTP vs protocole de jeu sur le même port
- `GET /health` → `{"status":"online"}`
- Matchmaking automatique : les joueurs sont mis en file et appairés dès que 2 sont prêts
- Le combat PvP est **arbitré tour par tour** côté serveur pour éviter la triche
- La reproduction échange les monstres entre joueurs via le serveur

## 🚀 Lancer le jeu

### Prérequis

- [Rust](https://rustup.rs/) (édition 2024)

### Client (TUI)

```bash
cargo run --bin monster-battle-tui
```

Par défaut, le client se connecte au serveur `monster-battle.darthoit.eu`. Pour utiliser un serveur local :

```bash
MONSTER_SERVER=localhost cargo run --bin monster-battle-tui
```

### Serveur

```bash
cargo run --bin monster-battle-server
```

Variables d'environnement optionnelles :

| Variable | Défaut | Description |
|---|---|---|
| `PORT` | `7878` | Port TCP (jeu + health check) |
| `MONSTER_SERVER` | `monster-battle.darthoit.eu` | Adresse du serveur (côté client) |

## 🎮 Comment jouer

### Premiers pas

1. Lancer le client TUI
2. **Créer un monstre** : choisir un type élémentaire parmi les 8, puis lui donner un nom
3. Ton monstre naît au niveau 1 avec des stats aléatoires basées sur son type

### Entraînement

- Depuis le menu, sélectionner **Entraîner** pour combattre un bot
- Les combats d'entraînement donnent **50 %** de l'XP d'un vrai combat
- Monter de niveau améliore les stats

### Combat PvP

- Sélectionner **Combat PvP** depuis le menu
- Le jeu te met en file d'attente sur le serveur
- Dès qu'un adversaire est trouvé, le **combat interactif** démarre :
  - Choisis tes attaques tour par tour
  - Les types comptent : exploite les faiblesses adverses !
  - Le vainqueur gagne de l'XP et un `win` — le perdant risque la **mort**

Le combat PvP est **arbitré par le serveur** — les deux joueurs interagissent en temps réel :

```
Joueur A                    Serveur                     Joueur B
   │                           │                            │
   ├── Queue ──────────────────►                            │
   │                           ◄── Queue ──────────────────┤
   ◄── Matched ────────────────┤── Matched ────────────────►
   ◄── CombatOpponent ─────────┤── CombatOpponent ─────────►
   │                           │                            │
   │       ┌── Boucle tour par tour ──┐                     │
   │       │                          │                     │
   ├── PvpAttackChoice ────────►      │  ◄── PvpAttackChoice ┤
   │       │   (attend les 2 choix)   │                     │
   ◄── PvpTurnResult ─────────┤── PvpTurnResult ──────────►
   │       │                          │                     │
   │       └── (répète jusqu'à K.O.) ─┘                     │
```

### Reproduction

- Sélectionner **Reproduction** depuis le menu
- Ton monstre est mis en file d'attente avec un autre joueur
- L'enfant hérite :
  - Des **stats** moyennées des deux parents (avec variation)
  - Du **type secondaire** de l'autre parent
  - De **traits génétiques** hérités, avec possibilité de **mutation**
  - D'une **génération** incrémentée
- Il faudra lui donner un nom !

### Vieillissement et mort

- Les monstres vieillissent en temps réel (**~30 jours** de durée de vie)
- Le trait **Longévité** ajoute 15 jours supplémentaires
- Un monstre mort rejoint le **cimetière** — il est consultable mais ne peut plus combattre
- Le chiffrement est retiré à la mort pour permettre l'archivage

### Contrôles

| Touche | Action |
|---|---|
| `↑` `↓` | Naviguer dans les menus / listes |
| `Enter` | Confirmer / Sélectionner |
| `Esc` | Retour / Annuler |
| `q` | Quitter |

## �️ Roadmap & Améliorations

### 🐉 Monstres & Statistiques

- [x] Système complet de monstre (UUID, nom, types, stats, niveau, XP, lignée, wins/losses)
- [x] 8 types élémentaires + Normal (table d'efficacité ×1.5 / ×0.5)
- [x] 4 stades de vie (Bébé / Jeune / Adulte / Vieux) avec multiplicateurs de stats
- [x] Vieillissement en temps réel (~30 jours de durée de vie)
- [x] Mort de vieillesse automatique
- [x] 8 traits génétiques héritables (Régénération, Évasion, Coup Critique+, Longévité, etc.)
- [x] Système de faim (4 niveaux, fenêtre de 12h, mort après 3 jours sans nourriture)
- [x] 5 types de nourriture (Baie, Viande, Poisson, Herbes, Gâteau) avec effets uniques
- [x] Buffs temporaires alimentaires (Viande → +15% ATK 1h, Poisson → +15% VIT 1h)
- [x] Système de bonheur (0–100, 5 niveaux, multiplicateurs stats & XP)
- [x] Système de lien / affinité (0–100, 5 niveaux, bonus survie & reproduction)
- [x] Événements aléatoires (6 types, 30% chance, cooldown 1h)
- [ ] Effets de statut en combat (brûlure, poison, paralysie, sommeil…)
- [ ] Système d'évolution (formes évoluées selon niveau/conditions)
- [ ] Objets tenus / équipement (bonus de stats ou effets spéciaux)
- [ ] Renommage de monstre après création
- [ ] Plus d'événements aléatoires (maladie, rencontres rares, événements saisonniers)
- [ ] Système météo / heure du jour affectant le gameplay

### ⚔️ Combat

- [x] Combat interactif style Pokémon (phases Intro → Choix → Exécution → Victoire/Défaite)
- [x] Files de messages stylisées avec animations
- [x] Système physique / spécial, coups critiques, précision/évasion, variance de dégâts
- [x] IA d'entraînement (priorise les attaques super efficaces)
- [x] Mode Docile (safe) et Sauvage (mort possible)
- [x] 3 attaques par élément (puissance 50/70/90, précision 100/90/75)
- [x] 4 attaques par monstre (1 universelle + 2-3 de type)
- [ ] Apprentissage d'attaques au level-up (remplacer une attaque existante)
- [ ] Plus de variété d'attaques (multi-coups, self-buffs, effets de terrain)
- [ ] Objets utilisables en combat (potions, buffs)
- [ ] Multijoueur local (2 joueurs même appareil)

### 🌐 PvP & Réseau

- [x] Combat PvP en temps réel via serveur relais
- [x] Matchmaking automatique par file d'attente
- [x] Arbitrage serveur tour par tour (anti-triche)
- [x] Gestion forfait / déconnexion (victoire par forfait)
- [x] Client WebSocket avec support TLS (wss://)
- [x] Protocole JSON avec préfixe de longueur
- [x] Health check HTTP (`GET /health`)
- [x] Vérification de version client/serveur
- [ ] Système de classement / ELO / ranking saisonnier
- [ ] Tournois organisés
- [ ] Rate limiting sur le serveur
- [ ] Authentification / comptes joueurs
- [ ] Validation des stats côté serveur (anti-triche renforcé)
- [ ] Reconnexion en cas de déconnexion mid-combat
- [ ] Heartbeat automatique serveur (ping/pong périodique)
- [ ] Logging structuré (tracing) et métriques serveur
- [ ] Arrêt gracieux du serveur (signal handling)

### 🧬 Reproduction

- [x] Reproduction entre monstres de joueurs différents via serveur
- [x] Héritage de type (50/50 primaire, 40% type secondaire)
- [x] Blending de stats (moyenne ± 15% variance)
- [x] Héritage de traits (50% par trait parent) + 15% mutation
- [x] Suivi de génération et de parenté
- [x] Bonus de lien sur les stats de l'enfant
- [ ] Échange / trade direct de monstres entre joueurs
- [ ] Visualisation de l'arbre généalogique / lignée

### 🎮 Mini-jeux

- [x] Morpion (Tic-Tac-Toe) avec IA à 3 niveaux de difficulté
- [x] Memory (paires de cartes, grilles variables selon difficulté)
- [x] Reflex / QTE (flèches directionnelles, 8/12/16 rounds)
- [x] RPS Élémentaire (triangles d'éléments, best-of 3/5/7)
- [x] Récompenses stats + XP selon difficulté et performance
- [x] Sélection du type de mini-jeu et de la difficulté
- [ ] Plus de mini-jeux (puzzle, course, rythme…)
- [ ] Musique dédiée par mini-jeu

### 🖥️ Client TUI

- [x] Interface terminal complète (ratatui + crossterm)
- [x] Sprites pixel-art en terminal (demi-blocs Unicode)
- [x] Animations de combat (tachyonfx : coalesce, sweep, glitch, glow, dissolve…)
- [x] Tous les écrans : menu, liste monstres, création, entraînement, PvP, reproduction, cimetière, aide, mini-jeux
- [x] Sélection et distribution de nourriture
- [x] Affichage des événements aléatoires
- [x] Modal de mise à jour (vérification version serveur)
- [x] Indicateur de connexion serveur en temps réel (🟢 / 🔴)
- [ ] Écran dédié détaillé pour un monstre (stats, attaques, arbre familial)
- [ ] Tableau d'efficacité des types in-game
- [ ] Revue du log de combat après la fin
- [ ] Écran de paramètres (volume, serveur, langue)
- [ ] Thème clair / sombre

### 📱 Client Android

- [x] Interface Bevy complète avec tous les écrans
- [x] Sprites pixel-art Bevy (RGBA8, scaling ×4, atlas caché)
- [x] Support tactile (navigation, scroll, tap)
- [x] Clavier IME pour la saisie de texte (workaround JNI)
- [x] Indicateur de connexion serveur avec vérification de version
- [x] Animations de combat (barre HP, flash d'attaque, dots d'attente)
- [x] Police personnalisée embarquée (emoji/unicode)
- [x] Réseau asynchrone via threads dédiés (résolution DNS JNI)
- [ ] Notifications push (monstre affamé, match trouvé en arrière-plan)
- [ ] Widget écran d'accueil (statut du monstre)
- [ ] Publication Play Store (métadonnées à compléter)
- [ ] Gestion mode hors-ligne (dégradation gracieuse)
- [ ] Nettoyage des écrans stub dans `screens.rs`

### 🎵 Audio

- [x] Moteur audio logiciel (rodio, thread-safe)
- [x] Synthétiseurs : Oscillateur (Sine/Square/Sawtooth/Triangle) + ADSR, KickDrum, Hihat, Snare
- [x] Mini-notation Strudel (séquences, groupes, alternances, repos, vitesse, polyphonie)
- [x] 7 pistes musicales (titre, combat, victoire, défaite, exploration, reproduction, cimetière)
- [x] 10 effets sonores (hit, crit, menu, level-up, mort, heal, match, fuite)
- [x] Contrôle volume + mute avec sauvegarde des préférences
- [x] Outil compositeur interactif (éditeur, preview live, save/load JSON)
- [ ] Thèmes de combat par élément de l'adversaire
- [ ] Plus de SFX (nourriture, reproduction, événements, attaques par type)
- [ ] Sliders de volume dans un écran paramètres

### 💾 Stockage & Sécurité

- [x] Chiffrement AES-256-GCM pour les monstres vivants
- [x] Clé dérivée de la machine (hostname + username + salt)
- [x] Monstres morts en clair (archivage consultable)
- [x] Protections anti-triche (anti-résurrection, validation intégrité)
- [x] Trait `MonsterStorage` générique (save, load, delete, list, export/import réseau)
- [ ] Synchronisation cross-device (cloud sync)
- [ ] Backup / restauration de sauvegarde complète
- [ ] Système de migration versionné pour le schéma Monster

### 🏗️ Infrastructure & Qualité

- [x] Workspace Cargo organisé (9 crates)
- [x] Makefile (desktop check/build/run, Android build/APK/install)
- [x] Pipeline de build Android (xbuild → AAB → APK, aapt2, zipalign, apksigner)
- [x] Script de génération d'icônes multi-densité
- [x] Dockerfile multi-stage pour le serveur
- [x] Viewer de sprites web ([sprites.html](web/sprites.html))
- [x] Tests unitaires core (génétique, entraînement, mini-jeux — 33 tests)
- [ ] Tests de combat, faim, bonheur, lien, événements
- [ ] Tests d'intégration (serveur + client end-to-end)
- [ ] CI/CD (GitHub Actions)
- [ ] Localisation / i18n (actuellement français uniquement)
- [ ] Documentation API en anglais
- [ ] Réduction des `unwrap()` non critiques

## �📄 Licence

MIT
