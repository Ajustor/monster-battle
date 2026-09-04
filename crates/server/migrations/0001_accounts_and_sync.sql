-- Comptes utilisateurs, identités OAuth et stockage serveur des monstres.
--
-- Le serveur fait autorité sur les monstres : les clients (TUI, mobile, web)
-- gardent un cache local et poussent leurs changements avec un numéro de
-- version pour détecter les conflits.

-- ── Comptes ───────────────────────────────────────────────────────────
CREATE TABLE users (
    id           UUID PRIMARY KEY,
    display_name TEXT        NOT NULL,
    avatar_url   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Une identité par fournisseur OAuth. Un même compte peut en lier plusieurs
-- (se connecter avec GitHub puis Google mène au même joueur si l'email colle).
CREATE TABLE identities (
    id               UUID PRIMARY KEY,
    user_id          UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider         TEXT        NOT NULL,
    provider_user_id TEXT        NOT NULL,
    email            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_user_id)
);

CREATE INDEX identities_user_id_idx ON identities (user_id);
CREATE INDEX identities_email_idx ON identities (lower(email)) WHERE email IS NOT NULL;

-- ── Sessions ──────────────────────────────────────────────────────────
-- Seul le SHA-256 du refresh token est stocké : une fuite de la base ne
-- permet pas de rejouer les sessions.
CREATE TABLE refresh_tokens (
    id           UUID PRIMARY KEY,
    user_id      UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash   BYTEA       NOT NULL UNIQUE,
    device_label TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ NOT NULL,
    revoked_at   TIMESTAMPTZ
);

CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens (user_id);

-- ── Device Authorization Grant (RFC 8628) ─────────────────────────────
-- Le TUI et le mobile ne peuvent pas héberger un redirect OAuth : ils
-- demandent un code, l'utilisateur l'approuve dans un navigateur, puis le
-- client échange son device_code contre des jetons.
CREATE TABLE device_auth_requests (
    id               UUID PRIMARY KEY,
    device_code_hash BYTEA       NOT NULL UNIQUE,
    user_code        TEXT        NOT NULL UNIQUE,
    device_label     TEXT,
    user_id          UUID        REFERENCES users (id) ON DELETE CASCADE,
    approved_at      TIMESTAMPTZ,
    consumed_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at       TIMESTAMPTZ NOT NULL
);

CREATE INDEX device_auth_requests_expires_at_idx ON device_auth_requests (expires_at);

-- État anti-CSRF d'un aller-retour OAuth, éventuellement rattaché à une
-- demande d'appairage device.
CREATE TABLE oauth_states (
    state           TEXT        PRIMARY KEY,
    provider        TEXT        NOT NULL,
    device_auth_id  UUID        REFERENCES device_auth_requests (id) ON DELETE CASCADE,
    redirect_to     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL
);

-- ── Monstres ──────────────────────────────────────────────────────────
-- `data` est le Monster sérialisé tel que le connaît monster-battle-core :
-- le schéma de jeu peut évoluer sans migration SQL. `version` est incrémenté
-- à chaque écriture et sert de jeton de concurrence optimiste.
CREATE TABLE monsters (
    id         UUID        PRIMARY KEY,
    owner_id   UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    data       JSONB       NOT NULL,
    version    BIGINT      NOT NULL DEFAULT 1,
    deleted    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index de pull incrémental : « donne-moi ce qui a changé depuis X ».
CREATE INDEX monsters_owner_updated_idx ON monsters (owner_id, updated_at);
