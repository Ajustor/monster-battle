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
- `combat::fight()` — moteur de combat tour par tour
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
- `NetMessage` — protocole JSON sérialisé avec préfixe de longueur (`Queue`, `Matched`, `CombatResult`, `BreedingPartner`, `Ping`/`Pong`, etc.)
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
- Le combat est exécuté côté serveur pour éviter la triche
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

## 📄 Licence

MIT
