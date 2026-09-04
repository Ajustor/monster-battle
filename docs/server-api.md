# API du serveur — comptes et synchronisation

Le serveur expose deux protocoles **sur un seul port** : le relais WebSocket
historique (combats PvP, reproduction) et une API HTTP pour les comptes et la
synchronisation multi-appareils. Les premiers octets de la connexion suffisent
à les distinguer, ce qui évite d'exiger deux ports en hébergement.

Sans `DATABASE_URL`, le serveur démarre **en relais seul** : le déploiement
existant continue de fonctionner sans base de données, `/health` répond, et
l'API n'est simplement pas montée.

## Configuration

| Variable | Obligatoire | Rôle |
|---|---|---|
| `PORT` | non (7878) | Port d'écoute, WebSocket et HTTP confondus |
| `DATABASE_URL` | non | Base Postgres. Absente = relais seul, sans comptes |
| `JWT_SECRET` | si `DATABASE_URL` | Clé de signature des jetons d'accès, ≥ 32 caractères |
| `PUBLIC_URL` | non | URL publique, base des callbacks OAuth et de la page d'appairage |
| `ALLOWED_ORIGINS` | non | Origines CORS autorisées, séparées par des virgules (site de l'arène) |
| `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` | non | Active la connexion GitHub |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | non | Active la connexion Google |

Les migrations sont embarquées dans le binaire et appliquées au démarrage.

## Authentification

Deux parcours mènent à une session, selon que le client dispose d'un
navigateur.

### Navigateur (site de l'arène)

```
GET /api/v1/auth/{provider}/start?redirect_to=<url>
      → 302 vers GitHub / Google
GET /api/v1/auth/{provider}/callback?code=…&state=…
      → 302 vers <url>#access_token=…&refresh_token=…&expires_in=…
```

Les jetons repartent dans le **fragment** de l'URL, qui n'est jamais transmis
au serveur ni journalisé. `redirect_to` n'est honoré que s'il pointe vers une
origine listée dans `ALLOWED_ORIGINS` — sans quoi le serveur servirait de
tremplin à redirection ouverte.

### Terminal et mobile (RFC 8628)

Le TUI et l'app mobile ne peuvent pas héberger de redirection OAuth. Ils
demandent un code, que le joueur approuve depuis n'importe quel navigateur :

```
POST /api/v1/auth/device/start   {"device_label": "TUI de salon"}
  → {"device_code": "…", "user_code": "PJ6C-89CX",
     "verification_uri": "https://…/api/v1/auth/device?code=PJ6C-89CX",
     "expires_in": 600, "interval": 3}

# Le joueur ouvre verification_uri et se connecte.

POST /api/v1/auth/device/token   {"device_code": "…"}
  → {"status": "pending", "interval": 3}          tant qu'il n'a pas approuvé
  → {"status": "ready", "access_token": …, …}     une fois approuvé
```

Le `user_code` évite les caractères ambigus (`0`/`O`, `1`/`I`/`L`) et se saisit
indifféremment avec ou sans tiret, en majuscules ou minuscules. Un `device_code`
ne donne des jetons **qu'une seule fois**.

Un joueur déjà connecté peut aussi approuver un appareil depuis l'arène :

```
POST /api/v1/auth/device/approve  {"user_code": "PJ6C-89CX"}   (jeton requis)
```

### Session

```
POST /api/v1/auth/refresh  {"refresh_token": "…"}   → nouvelle paire de jetons
POST /api/v1/auth/logout   {"refresh_token": "…"}   → révoque l'appareil
GET  /api/v1/me                                      (jeton requis)
GET  /api/v1/auth/providers                          → fournisseurs configurés
```

Le jeton d'accès est un JWT court (1 h) et non révocable ; c'est le refresh
token, stocké **haché** en base, qui porte la session révocable. Il est
**tourné à chaque rafraîchissement** : un jeton volé cesse de fonctionner dès
que le client légitime rafraîchit sa session.

Une tolérance de 60 s est appliquée sur l'expiration du jeton d'accès, les
horloges des clients n'étant pas toujours réglées.

## Synchronisation des monstres

Le serveur fait autorité. Chaque monstre porte un `version` incrémenté à chaque
écriture, qui sert de jeton de concurrence optimiste.

```
GET  /api/v1/monsters?since=<rfc3339>   → {monsters: [...], server_time}
GET  /api/v1/monsters/{id}
DELETE /api/v1/monsters/{id}
POST /api/v1/sync
```

`POST /api/v1/sync` fait l'aller-retour complet :

```jsonc
// Requête
{
  "since": "2026-09-04T07:00:00Z",        // point de reprise, null au premier appel
  "changes": [
    { "monster": { /* Monster sérialisé */ }, "base_version": 3 }
  ],
  "deleted": ["<uuid>"]
}

// Réponse
{
  "results": [                             // sort de chaque push, dans l'ordre
    { "status": "applied",  "id": "…", "version": 4 },
    { "status": "conflict", "id": "…", "server": { /* copie serveur */ } },
    { "status": "rejected", "id": "…", "problems": ["niveau 9999 hors des bornes 1–100"] }
  ],
  "monsters": [ /* tout ce qui a changé depuis `since` */ ],
  "server_time": "2026-09-04T07:27:36Z"    // à repasser au prochain appel
}
```

**Conventions**

- `base_version` absent = création. Un `base_version` sur un monstre inconnu du
  serveur est un conflit, pas une création : le monstre a pu être supprimé
  ailleurs.
- Une suppression laisse une **pierre tombale** (`deleted: true`, sans `monster`)
  pour que les autres appareils propagent l'effacement.
- Un monstre appartenant à un autre joueur est traité comme inexistant : on ne
  révèle rien de la collection d'autrui.
- Au maximum 200 modifications par appel.

## Validation anti-triche

Le serveur ne peut pas rejouer l'histoire d'un monstre, mais il refuse les états
manifestement impossibles avant qu'un monstre n'entre dans l'arène : niveau hors
de 1–100, statistique de base hors de 1–255, PV supérieurs au maximum, XP
au-delà du palier de passage de niveau, traits en double, date de naissance dans
le futur, lignée incohérente (génération > 0 sans parents, un seul parent).

C'est une barrière contre l'édition grossière de sauvegarde, pas une preuve
d'authenticité de chaque point d'XP.

## Développement local

```bash
make server-db     # Postgres dans Docker
make server-run    # serveur sur http://localhost:7878
make server-test   # tests unitaires + intégration sur la base
```

Les tests d'intégration ne s'exécutent que si `TEST_DATABASE_URL` est défini ;
sinon ils s'ignorent silencieusement. Chacun travaille sur son propre schéma
Postgres, ce qui les rend indépendants et parallélisables.
