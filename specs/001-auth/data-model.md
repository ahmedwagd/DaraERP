# Data Model — 001-auth: Authentication & Session Management

## Existing Tables (from 001_init.sql)

These tables were created during foundation setup and are reused by this feature.

### `users`

```sql
CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY,       -- UUID
    email           TEXT NOT NULL UNIQUE,
    name            TEXT,
    avatar_url      TEXT,
    password_hash   TEXT NOT NULL,          -- argon2id hash
    role            TEXT NOT NULL DEFAULT 'USER',  -- ADMIN | MANAGER | USER
    language        TEXT NOT NULL DEFAULT 'en',     -- en | ar
    department      TEXT,
    team            TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Auth-relevant fields**: `email`, `password_hash`, `role`, `language`, `is_active`.

**Auth constraints**:
- Only active users (`is_active = 1`) can authenticate.
- `password_hash` is generated via argon2id; never returned to the frontend.
- For Feature 001 validation, at least one active ADMIN user must exist before login testing begins.
  Account creation and management are outside this feature except for a development-only fixture
  or seed path (see plan.md "Initial Admin Account Prerequisite").

### `refresh_tokens`

```sql
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id          TEXT PRIMARY KEY,           -- UUID
    token_hash  TEXT NOT NULL,              -- SHA-256 of opaque token string
    family_id   TEXT NOT NULL,              -- UUID grouping tokens from one login
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TEXT NOT NULL,              -- ISO-8601
    is_revoked  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);
```

**Auth constraints**:
- `token_hash` is the SHA-256 of the opaque refresh token (never stored raw).
- `family_id` groups all tokens from a single login session. All tokens in a family can be revoked at once (theft detection).
- `is_revoked = 1` marks a single token as used (rotation) or the whole family as compromised.
- `expires_at` is checked on every refresh attempt. Expired tokens are treated as invalid.

### `audit_logs`

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id          TEXT PRIMARY KEY,           -- UUID
    entity_type TEXT NOT NULL,              -- 'USER' | 'SESSION' | etc.
    entity_id   TEXT NOT NULL,              -- user UUID
    action      TEXT NOT NULL,              -- LOGIN | LOGOUT | LOGIN_FAILED | TOKEN_THEFT_DETECTED
    changes     TEXT,                       -- JSON with details (e.g., {"ip": "local"})
    user_id     TEXT NOT NULL,
    user_email  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Auth audit events**:
| Action | entity_type | entity_id | When |
|--------|------------|-----------|------|
| `LOGIN` | USER | user UUID | Successful login |
| `LOGOUT` | USER | user UUID | Explicit logout |
| `LOGIN_FAILED` | USER | user UUID (if known) or 'unknown' | Failed login attempt |
| `TOKEN_THEFT_DETECTED` | USER | user UUID | Stale token reuse detected |

## In-Memory State (AppState)

Not persisted to SQLite. Held in `Mutex<Option<Session>>` on `AppState`.

```rust
pub struct Session {
    pub user_id: String,
    pub role: String,
    pub language: String,
}
```

## OS Keychain Entries

| Service | Account | Value | When Written |
|---------|---------|-------|-------------|
| `daraerp` | `jwt-secret` | 256-bit random hex string | First app launch |
| `daraerp` | `refresh-token` | Base64 opaque token | Login, refresh |

## State Transitions

### User Account Lifecycle
```
[Created] ──► Active (is_active=1) ──► [Deactivated] (is_active=0)
                     │
                     └── Authenticates ← password_hash verified
```

### Refresh Token Family Lifecycle
```
Login ──► Family Created (family_id=X)
              │
              └── Token0 issued (is_revoked=0)
                       │
                  Refresh ──► Token0 revoked (is_revoked=1) + Token1 issued
                                    │
                               Refresh ──► Token1 revoked + Token2 issued
                                                 │
                                     ┌───────────┴────────────┐
                                     │                        │
                              Normal flow               Theft detected:
                              continues...              Token0 reused → all tokens
                                                        in family revoked
```

### Session State (AppState)
```
[None] ──login()──► Some(Session{user_id, role, lang})
                         │
                         ├── refresh_session() → updates Session
                         │
                         └── logout() → revoke family + [None]
```
