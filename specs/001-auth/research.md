# Research — 001-auth: Authentication & Session Management

## Decision 1: Password Hashing

**Decision**: Use the `argon2` crate with `Argon2id` variant, `PasswordHasher` API.

**Rationale**:
- Constitution mandates argon2. AGENTS.md lists `argon2` as a planned dependency.
- `argon2` crate is the standard Rust implementation (maintained, audited).
- `Argon2id` is the recommended variant (hybrid of Argon2d and Argon2i, resistant to both side-channel and GPU attacks).
- Use `PasswordHasher::default()` which applies OWASP minimum parameters (m=19456, t=2, p=1).

**Alternatives considered**:
- `bcrypt` crate: Weaker than argon2, not in planned dependencies. Rejected.
- `scrypt` crate: Viable but not in planned dependencies; argon2 is preferred for greenfield. Rejected.
- Bare `sha2`: Not suitable for password hashing (no salt, too fast). Rejected.

## Decision 2: Access Credentials (JWT)

**Decision**: Use the `jsonwebtoken` crate for short-lived access credentials. Encode `user_id`, `role`, `language`, `exp`, `iat`. Verify with HS256 using a 256-bit random secret stored in the OS keychain.

**Rationale**:
- Constitution specifies JWT. AGENTS.md lists `jsonwebtoken` as a planned dependency.
- `jsonwebtoken` crate is the most-used Rust JWT library.
- HS256 is sufficient for a local desktop app (no network transmission; the JWT is passed to backend commands in-process via Tauri IPC).
- Secret stored in OS keychain via `keyring` crate. Generated on first app launch with `rand::thread_rng()`.

**Alternatives considered**:
- Session-based (random session_id in DB): Simpler but constitution/AGENTS.md explicitly specify JWT. Rejected for compliance.
- PASETO tokens: More modern standard, but no stable Rust crate with the ecosystem support of `jsonwebtoken`. Rejected.
- RS256 (asymmetric): Overkill for local desktop. Adds key management complexity. Rejected.

## Decision 3: Refresh Token Strategy

**Decision**: Opaque random strings (32 bytes, base64-encoded) with SHA-256 hashes stored in SQLite. Family-based rotation with theft detection.

**Rationale**:
- Spec requires rotation on every use and theft detection via family revocation.
- Family-based approach (RFC 6819 section 5.2.2 pattern): Each login creates a `family_id`. Each refresh within that family generates a new token and invalidates the old one. If an already-invalidated token is reused, the entire family is revoked.
- The opaque token is returned to the frontend and stored in the OS keychain. The SHA-256 hash is persisted in the `refresh_tokens` table.
- `sha2` crate (planned dependency) for hashing; `rand` crate (planned dependency) for random generation.

**Alternatives considered**:
- JWT refresh tokens: Would need to store revocation list or use short-lived JWT + refresh. Family tracking with opaque strings is simpler and more secure for theft detection. Rejected.
- Single refresh token (no rotation): Violates spec requirement for rotation. Rejected.

## Decision 4: OS Keychain Access

**Decision**: Use the `keyring` crate for cross-platform keychain access.

**Rationale**:
- Constitution: "use an OS keychain/keyring approach." AGENTS.md: "secure storage" not `tauri-plugin-store`.
- `keyring` crate provides a unified API for:
  - macOS: Keychain Services
  - Windows: Credential Manager
  - Linux: Secret Service (via `dbus-secret-service`) or `libsecret`
- Entries stored under service name `daraerp` with account keys: `jwt-secret`, `refresh-token`.
- Keychain calls are synchronous; they should be called once during app startup (load JWT secret) and from auth commands (store/get refresh tokens). Minimal performance impact.

**Alternatives considered**:
- `tauri-plugin-store`: Constitution explicitly forbids this for secrets. Rejected.
- `tauri-plugin-keystore`: Not a stable Tauri community plugin for v2. Rejected.
- Manual platform-specific FFI: Unnecessary complexity; `keyring` crate covers all targets. Rejected.

## Decision 5: Session State Management

**Decision**: Store current user session info (`user_id`, `role`, `language`) in `AppState` behind a `Mutex<Option<Session>>`. Set on login/refresh, cleared on logout.

**Rationale**:
- Single-user desktop app — no concurrent sessions.
- Commands read session from `AppState` to enforce role checks without requiring the frontend to pass credentials on every call.
- Mutex protects against Tauri's multi-threaded command dispatch.

**Alternatives considered**:
- Pass JWT on every command: More explicit but adds boilerplate to every command and every frontend call. Rejected for usability.
- Tauri command middleware/interceptor: Tauri 2 doesn't have a built-in command middleware pattern. Rejected as unavailable.

## Decision 6: Audit Logging

**Decision**: Reuse the existing `audit_logs` table from `001_init.sql`. Call `audit_log()` function after each tracked event.

**Rationale**:
- Table already exists with fields: `entity_type`, `entity_id`, `action`, `changes` (JSON), `user_id`, `user_email`, `created_at`.
- Audit events for auth: `LOGIN`, `LOGOUT`, `LOGIN_FAILED`, `TOKEN_THEFT_DETECTED`.
- `audit.rs` currently empty; will be populated with `audit_log()` function.

**Alternatives considered**:
- Separate auth-specific audit table: Unnecessary duplication. Rejected.
- File-based audit log: Harder to query from UI. Rejected.

## Decision 7: Frontend i18n

**Decision**: `react-i18next` with `i18next`, namespace `auth`, languages `en` and `ar`.

**Rationale**:
- AGENTS.md lists `react-i18next` and `i18next` as planned dependencies.
- Separate namespace per feature keeps translation files manageable.
- Language preference stored per-user in the `users` table; loaded on login and stored in session state.

**Alternatives considered**:
- Custom i18n solution: Less maintainable, no plurals/interpolation support. Rejected.
- `react-intl`: Heavier, less commonly used with Tauri projects. Rejected.
