---
description: "Task list for 001-auth: Authentication & Session Management"
---

# Tasks: Authentication & Session Management

**Input**: Design documents from `specs/001-auth/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: No test tasks are included. Tests are not requested for this feature. Use the
project-approved checks only (`cargo check`, `cargo clippy`, `npx tsc --noEmit`, `npm run lint`).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing
of each story. Foundational phase includes shared infrastructure that blocks all user stories.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Rust backend**: `src-tauri/src/`
- **Frontend**: `src/`

---

## Phase 1: Setup — Install Dependencies

**Purpose**: Add required Rust and frontend dependencies

- [ ] T001 Add Rust auth dependencies to `src-tauri/Cargo.toml`: argon2, jsonwebtoken, sha2, rand,
      keyring (plus any of uuid, chrono, serde, serde_json not already present)
- [ ] T002 [P] Install frontend dependencies via `npm install`: react-router, react-i18next, i18next

---

## Phase 2: Foundational — Core Auth & Shared Infrastructure

**Purpose**: Core auth modules, secure storage, models/queries, audit, i18n, Session/AppState wiring.
MUST complete before any user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Create development-only admin account fixture in `src-tauri/src/seed.rs`:
      - Gated behind `#[cfg(debug_assertions)]` from the start; never runs in production builds.
      - Inserts one ADMIN user with an argon2-hashed password only when the `users` table is empty.
      - Called from `src-tauri/src/db.rs` after migrations when no admin exists and `cfg(debug_assertions)`.
      - Clearly documented as temporary until Feature 002 formalises user creation.
      - Does not implement general user management.
- [ ] T004 Implement `audit_log()` in `src-tauri/src/audit.rs`:
      - Supports actions: `LOGIN`, `LOGOUT`, `LOGIN_FAILED`, `TOKEN_THEFT_DETECTED`.
      - Accepts user_id, user_email, entity_type, entity_id, optional JSON changes.
      - Returns `AppError` consistent with the error convention; never panics.
- [ ] T005 [P] Implement auth-related model/query helpers in `src-tauri/src/models/mod.rs`:
      - `find_user_by_email(email) -> Result<User, AppError>`
      - `find_user_by_id(id) -> Result<User, AppError>`
      - `insert_refresh_token(user_id, family_id, token_hash, expires_at) -> Result<(), AppError>`
      - `find_refresh_token_by_hash(token_hash) -> Result<RefreshToken, AppError>`
      - `revoke_refresh_token(id) -> Result<(), AppError>`
      - `revoke_family(family_id) -> Result<(), AppError>`
      - `get_active_family_id(user_id) -> Result<Option<String>, AppError>`
      - All return typed errors; never unwrap.
- [ ] T006 [P] Implement secure store in `src-tauri/src/auth/secure_store.rs`:
      - `load_jwt_secret() -> Result<String, AppError>` — generate 256-bit random secret on first
        run and store in keyring; load from keyring on subsequent runs.
      - `get_refresh_token() -> Result<String, AppError>` — read raw renewal credential from keyring.
      - `set_refresh_token(token: &str) -> Result<(), AppError>` — store raw renewal credential in keyring.
      - `clear_refresh_token() -> Result<(), AppError>` — remove renewal credential from keyring.
      - Uses `keyring` crate internally. Never exposes raw secrets to the frontend.
- [ ] T007 Implement `Session` struct in `src-tauri/src/auth/session.rs`:
      - Fields: `user_id: String`, `role: String`, `language: String`, `family_id: String`.
      - Update `src-tauri/src/lib.rs` to add `pub session: Mutex<Option<Session>>` and
        `pub jwt_secret: String` fields to `AppState`.
      - Update `src-tauri/src/db.rs` `init_db()` return to propagate `jwt_secret` from secure store
        or load it in `lib.rs` setup and pass to `AppState`.
      - Keep database connection in AppState as established by foundation.
- [ ] T008 Implement core auth functions in `src-tauri/src/auth/mod.rs`:
      - `hash_password(password: &str) -> Result<String, AppError>` — argon2id hash.
      - `verify_password(password: &str, hash: &str) -> Result<bool, AppError>` — argon2id verify.
      - `issue_tokens(user_id: &str, role: &str, jwt_secret: &str) -> Result<(String, String, String, DateTime<Utc>), AppError>`
        — generate JWT access token (exp=15min, sub=user_id, role, lang) and opaque refresh token
        (32-byte random, base64); return (jwt, refresh_token_raw, family_id, refresh_expiry).
      - `verify_jwt(token: &str, jwt_secret: &str) -> Result<Claims, AppError>` — decode and
        validate JWT, return claims (user_id, role, language).
      - `rotate_refresh_token(conn: &Connection, raw_token: &str, jwt_secret: &str) -> Result<(String, String, String, String, DateTime<Utc>), AppError>`
        — find token by SHA-256 hash; if not found → return invalid credentials; if found and
        expired → clear session, return expired error; if found and already revoked (is_revoked=1)
        → suspect theft: call `revoke_family`, write `TOKEN_THEFT_DETECTED` audit, clear secure
        store, clear session, return invalid credentials error; if found, active, not expired →
        revoke this token only, issue new token in same family, return (new_jwt, new_refresh_raw,
        family_id, user_id, new_refresh_expiry).
      - `revoke_family_by_user(conn: &Connection, family_id: &str) -> Result<(), AppError>`
        — revoke all tokens in family.
      - `guard_role(user_role: &str, allowed_roles: &[&str]) -> Result<(), AppError>`
        — return `PERMISSION_DENIED` if role not in allowed set.
- [ ] T009 [P] Set up i18n in `src/i18n/index.ts` with `i18next` and `react-i18next`, configure
      `en` and `ar` namespaces with auth translations in `src/i18n/en.json` and `src/i18n/ar.json`

**Checkpoint**: Foundation ready — `audit_log()` works, model queries work, secure store works,
`Session`/`AppState` wired, core auth functions complete with correct theft detection, `guard_role`
implemented, i18n configured. User story implementation can now begin.

---

## Phase 3: User Story 1 — Login (Priority: P1) 🎯 MVP

**Goal**: User can log in with email/password; invalid attempts show safe errors; language
selector works; audit events are written.

**Independent Test**: Open app → see login form → enter valid credentials → dashboard loads.
Enter invalid credentials → error stays on login screen.

- [ ] T010 [US1] Implement `login` Tauri command in `src-tauri/src/commands/auth.rs`:
      - Validate email format and password non-empty.
      - `find_user_by_email` — if not found, return `AUTH_INVALID_CREDENTIALS`, write `LOGIN_FAILED` audit.
      - `verify_password` — if fails, return `AUTH_INVALID_CREDENTIALS`, write `LOGIN_FAILED` audit.
      - `issue_tokens` — generate JWT + refresh token.
      - `insert_refresh_token` — persist refresh token verifier (SHA-256 hash).
      - `set_refresh_token` — store raw refresh token in secure store.
      - Set `AppState.session = Some(Session { user_id, role, language, family_id })`.
      - Write `LOGIN` audit event.
      - Return `{ id, name, email, role, language }` (never password_hash or raw tokens).
- [ ] T011 [US1] Register `commands::auth` module and `login` command in `src-tauri/src/lib.rs`
      `generate_handler![]`
- [ ] T012 [US1] Create `src/services/auth.ts` with typed `login(email, password) -> User`
      function using `invokeCommand` wrapper — define TypeScript `User` and response types
- [ ] T013 [US1] Create `src/pages/Login.tsx` with email/password form, language selector,
      validation feedback, i18n bindings, and error display
- [ ] T014 [US1] Create `src/contexts/AuthContext.tsx` with user state, `login()` function that
      calls `src/services/auth.ts`, language preference, and provides `user` and `login` to children

**Checkpoint**: User Story 1 complete — login works end-to-end, invalid credentials show safe
errors, audit events recorded, language switching works on login form.

---

## Phase 4: User Story 2 — Protected Layout & Session Persistence (Priority: P1)

**Goal**: After login, user sees protected layout with role-appropriate nav. App restart
silently restores session without login screen. Logout works as safe workflow.

**Independent Test**: Log in → dashboard with sidebar appears. Close app → reopen →
dashboard loads without login form. Click logout → login screen appears → reopen → login
screen appears.

- [ ] T015 [US2] Implement `refresh_session` Tauri command in `src-tauri/src/commands/auth.rs`:
      - `get_refresh_token` from secure store; if absent → return `AUTH_INVALID_CREDENTIALS`.
      - `rotate_refresh_token` — handles theft detection, expiry, rotation internally (see T008).
      - On success: `set_refresh_token` with new raw token, update `AppState.session`, return user info.
      - On theft detected: `clear_refresh_token`, clear `AppState.session`, return `AUTH_INVALID_CREDENTIALS`.
- [ ] T016 [US2] Implement `get_current_user` Tauri command in `src-tauri/src/commands/auth.rs`:
      - Read `AppState.session`. If `None` → return null.
      - `find_user_by_id(session.user_id)`. If not found → return null.
      - Return `{ id, name, email, role, language }`.
- [ ] T017 [US2] Implement `logout` Tauri command in `src-tauri/src/commands/auth.rs`:
      - Read `family_id` from `AppState.session`. If `None` → already logged out, return null.
      - `revoke_family_by_user(family_id)` in DB.
      - `clear_refresh_token` from secure store.
      - Clear `AppState.session` to `None`.
      - Write `LOGOUT` audit event.
      - If any step fails: still clear in-memory session; return a safe error; do not leave the
        UI showing a valid logged-in state. Log the error server-side.
- [ ] T018 [US2] Register `refresh_session`, `get_current_user`, `logout` in
      `src-tauri/src/lib.rs` `generate_handler![]`
- [ ] T019 [US2] Add `refreshSession()`, `getCurrentUser()`, `logout()` to `src/services/auth.ts`
- [ ] T020 [US2] Create `src/components/Layout.tsx` with sidebar navigation (role-filtered),
      header showing user name/role/language, and main content outlet
- [ ] T021 [US2] Create `src/components/ProtectedRoute.tsx` that redirects to login when no
      valid session exists
- [ ] T022 [US2] Create `src/pages/Dashboard.tsx` placeholder protected page with welcome message
      and role badge
- [ ] T023 [US2] Update `src/App.tsx` with `AuthProvider`, router configuration (login route,
      protected routes), and auth-check redirect logic
- [ ] T024 [US2] Update `AuthContext` in `src/contexts/AuthContext.tsx` to call `refresh_session`
      on app mount for silent session restore

**Checkpoint**: User Stories 1 AND 2 complete — login, protected layout, silent session
restore, and safe logout workflow all functional.

---

## Phase 5: User Story 3 — Session Renewal Rotation & Theft Detection (Priority: P2)

**Goal**: Every refresh invalidates the previous renewal credential. Reuse of an already-
invalidated credential revokes the entire credential family.

**Independent Test**: Log in → capture refresh token → refresh session (rotation occurs) →
use old captured token → entire family revoked, user forced to re-login.

- [ ] T025 [US3] Verify and test theft detection path in `rotate_refresh_token` in
      `src-tauri/src/auth/mod.rs` (logic was implemented in T008; this task validates the
      correct behavior):
      - A presented renewal credential whose hash is found in the DB with `is_revoked=1`
        triggers full family revocation, secure store clear, session clear, and
        `TOKEN_THEFT_DETECTED` audit event.
      - A presented renewal credential whose hash is NOT found returns `AUTH_INVALID_CREDENTIALS`.
      - A presented renewal credential found, active, and not expired performs normal rotation
        (revoke old, issue new in same family).
- [ ] T026 [US3] End-to-end verification: run through quickstart scenario 9 (theft detection)
      as manual validation

**Checkpoint**: User Stories 1, 2, AND 3 complete — rotation and theft detection working correctly.

---

## Phase 6: User Story 4 — Role-Based Access Control (Priority: P2)

**Goal**: Role-authorization helper guards protected auth/session operations. Layout shows
role-appropriate navigation. Document role policy for future feature commands.

**Independent Test**: Log in as USER → attempt an ADMIN-only auth operation →
PERMISSION_DENIED error. Log in as ADMIN → same operation succeeds.

- [ ] T027 [US4] Apply `guard_role` to auth commands where meaningful in
      `src-tauri/src/commands/auth.rs`:
      - If Feature 001 has any admin-only auth operations, apply the guard.
      - If all auth operations are multi-role, document that guard is available but no auth
        command requires ADMIN-only scope in this feature.
      - Add a block comment in `commands/auth.rs` documenting the required role policy for
        future feature commands:
        ```
        // Role policy (for future use by non-auth commands):
        //   ADMIN  → users, settings, full access
        //   MANAGER → operational mutations (invoices, customers, products)
        //   USER   → read-only
        ```
      - Do not create fake business commands to demonstrate role enforcement.
- [ ] T028 [US4] Add role-based navigation filtering in `src/components/Layout.tsx`:
      show/hide sidebar nav items based on role. Planned sections may appear as disabled
      placeholders but must not imply unimplemented features exist.
- [ ] T029 [US4] Add role badge display in `src/pages/Dashboard.tsx` and user header in
      `src/components/Layout.tsx`

**Checkpoint**: All user stories complete — role-based access enforced at backend and
reflected in UI navigation. Role policy documented for future features.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final integration, validation, and cleanup

- [ ] T030 Run `npx tsc --noEmit` and fix all TypeScript errors
- [ ] T031 Run `npm run lint` and fix all lint errors
- [ ] T032 Run `cd src-tauri && cargo check` and fix all Rust errors
- [ ] T033 Run `cd src-tauri && cargo clippy -- -D warnings` and fix all clippy warnings and
      deny-level lint violations
- [ ] T034 Verify that the development-only admin seed in `src-tauri/src/seed.rs` is correctly
      gated behind `#[cfg(debug_assertions)]`. Verify no default password, hard-coded credential,
      or dev admin path is active in production builds
- [ ] T035 Run quickstart.md validation scenarios manually:
      - Login with valid credentials (scenario 2)
      - Login with invalid credentials (scenario 3)
      - Session persistence on app restart (scenario 5)
      - Safe logout workflow (scenario 6)
      - Expired session handling (scenario 7)
      - RBAC via auth-owned operations (scenario 8 — limited to Feature 001 scope)
      - Theft detection (scenario 9)
      - Arabic RTL login (scenario 10)
      - Note: RBAC validation on Customers/Invoices/Products/Settings commands is deferred to
        those feature implementations

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational completion
  - US1 (Login): No dependency on other user stories
  - US2 (Session + Layout): Depends on US1 (needs working login for session flow)
  - US3 (Theft Detection): Depends on US2 (theft detection is in the refresh flow)
  - US4 (RBAC): Depends on US2 (needs session + layout for role display)
- **Polish (Phase 7)**: Depends on all user stories

### Within Each User Story

- Backend commands before frontend wiring
- Services before pages
- Story complete before moving to next priority

### Parallel Opportunities

- Phase 1: T001 and T002 are parallel
- Phase 2: T005 and T006 are parallel; all others are sequential within same-file boundaries
  (T003 → T004 → (T005 ∥ T006) → T007 → T008 → T009)
- Phase 3: T010 (command def) → T011 (registration) → T012 (service) ∥ T013 (page) → T014 (context)
- Phase 4: T015 → T016 → T017 (ordered, same file) → T018 (registration) → T019 (service) →
  T020 ∥ T021 ∥ T022 (parallel frontend) → T023 → T024
- Phase 5: T025 → T026 (sequential)
- Phase 6: T027 → T028 ∥ T029 (parallel frontend)
- Phase 7: T030 ∥ T032 (parallel build checks), T031, T034, T035 sequential

---

## Parallel Examples

### Phase 2 Foundational

```bash
# Sequential (same file boundaries):
T003 - seed.rs
T004 - audit.rs
# Parallel:
T005 - models/mod.rs
T006 - auth/secure_store.rs
# Sequential:
T007 - auth/session.rs + lib.rs AppState
T008 - auth/mod.rs (core functions + guard_role)
T009 - i18n setup (frontend)
```

### Phase 3 User Story 1 (Login)

```bash
# Sequential (command def → registration):
T010 - commands/auth.rs
T011 - lib.rs generate_handler![]
# Parallel (different files, no dependencies):
T012 - services/auth.ts
T013 - pages/Login.tsx
# Sequential:
T014 - contexts/AuthContext.tsx
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Login)
4. **STOP and VALIDATE**: Test login flow manually
5. Proceed to remaining user stories

### Incremental Delivery

1. Phase 1 + Phase 2 → Foundation ready
2. Add User Story 1 (Login) → Test independently (MVP!)
3. Add User Story 2 (Session/Layout) → Test independently
4. Add User Story 3 (Theft Detection) → Test independently
5. Add User Story 4 (RBAC) → Test independently
6. Phase 7: Polish and final validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Do not run `cargo test`, `npm test`, or any test command unless explicitly requested
- Use only project-approved checks: `cargo check`, `cargo clippy`, `npx tsc --noEmit`,
  `npm run lint`
- The development-only admin account (T003) is gated behind `#[cfg(debug_assertions)]` from
  the start; it is temporary until Feature 002 formalises user creation
- Theft detection logic in T008 checks whether the *presented* renewal credential (by stored
  SHA-256 hash) is already revoked — this is correct. Normal rotation revokes the old token
  and creates a new one in the same family; the old token's `is_revoked=1` triggers theft
  only if that specific old token is presented again
- Role authorization for Customer/Invoice/Product/Settings commands is deferred to those
  feature implementations
