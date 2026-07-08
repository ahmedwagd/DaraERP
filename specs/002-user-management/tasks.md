---
description: "Task list for 002-user-management: User Management"
---

# Tasks: User Management

**Input**: Design documents from `specs/002-user-management/`

**Prerequisites**: plan.md, spec.md. All infrastructure from Feature 001 (auth, session, audit, models) is already in place.

**Tests**: No test tasks are included. Use the project-approved checks only (`cargo check`, `cargo clippy`, `npx tsc --noEmit`, `npm run lint`).

**No new dependencies**: All crates and npm packages required are already installed from Feature 001.

**Organization**: Tasks are grouped by layer. Foundational phase (audit + models) blocks all command tasks. Commands are grouped by user story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Rust backend**: `src-tauri/src/`
- **Frontend**: `src/`

---

## Phase 1: Foundational — Audit Actions & Model Queries

**Purpose**: Shared constants and query helpers that all user-management commands depend on.

**⚠️ CRITICAL**: No command work can begin until this phase is complete.

- [x] T001 Add audit action constants to `src-tauri/src/audit.rs`:
      - `USER_CREATED`, `USER_UPDATED`, `USER_ACTIVATED`, `USER_DEACTIVATED`.
      - Add them inside the existing `pub mod actions` block.
      - These are `pub const &str` values matching the existing pattern (LOGIN, LOGOUT, etc.).
- [x] T002 [P] Implement user CRUD query helpers in `src-tauri/src/models/mod.rs`:
      - `insert_user(conn, id, email, name, password_hash, role, language) -> Result<User, AppError>`
        — insert row, then SELECT the newly inserted user by ID and return it.
        Map SQLite ConstraintViolation to `CONFLICT` error.
      - `list_users(conn, role_filter, active_filter) -> Result<Vec<User>, AppError>`
        — support optional `role: Option<&str>` and `active: Option<bool>` filters;
        build SQL dynamically with `ORDER BY created_at DESC`.
      - `update_user(conn, id, email, name, role, language) -> Result<User, AppError>`
        — update only provided fields; always update `updated_at = datetime('now')`;
        after update, SELECT the updated user by ID and return it.
        Map ConstraintViolation to `CONFLICT`.
      - `set_user_active(conn, id, is_active) -> Result<User, AppError>`
        — update is_active + updated_at; return `NOT_FOUND` if no rows affected;
        after update, SELECT the updated user by ID and return it.
      - `count_active_admins(conn) -> Result<i64, AppError>`
        — `SELECT COUNT(*) FROM users WHERE role = 'ADMIN' AND is_active = 1`.
      - `revoke_all_refresh_tokens_for_user(conn, user_id) -> Result<(), AppError>`
        — `UPDATE refresh_tokens SET is_revoked = 1 WHERE user_id = ?1`.
      - All query helpers must accept `&Connection` (which also works inside transactions
        via `rusqlite::Transaction`'s Deref to `Connection`).
      - All return typed `AppError`; never unwrap or expect.
      - `find_user_by_id` already exists from Feature 001; reuse it.

**Checkpoint**: Audit constants ready, all model queries ready. Commands can now be built on top.

---

## Phase 2: User Story 1 — Create User (Priority: P1)

**Goal**: ADMIN creates a new user account. Duplicate email, invalid input, and non-ADMIN access are rejected.

**⚠️ Note**: All user-management commands share a `require_admin` helper (defined in T003). This
helper locks `AppState.session`, checks for an active session (→ `UNAUTHORIZED`), checks the role is
ADMIN (→ `PERMISSION_DENIED`), and returns the acting admin's `user_id`. It must release the session
lock before returning. Each mutation command (create, update, set_active) must then look up the
acting admin user by ID inside the DB transaction before writing audit events.

- [x] T003 [US1] Create `src-tauri/src/commands/users.rs` with `UserResponse` DTO and `require_admin`
      helper function:
      - `UserResponse` fields: `id`, `email`, `name`, `role`, `language`, `is_active`,
        `created_at`, `updated_at`. Never includes `password_hash`.
      - `require_admin(state) -> Result<String, AppError>`: locks session → checks `Some` →
        calls `auth::guard_role(&session.role, &["ADMIN"])` → returns `user_id.clone()`.
        **Must release session lock before returning** so caller can acquire DB lock independently.
      - **Acting admin audit pattern**: `require_admin` returns only the user ID. Every mutation
        command must call `models::find_user_by_id` inside the DB transaction to derive the acting
        admin's email before writing audit. Audit events must include acting admin ID, acting admin
        email, target user ID, action, and safe changes JSON. Never include passwords or hashes.
- [x] T004 [US1] Implement `create_user` command in `src-tauri/src/commands/users.rs`:
      - Input: `email: String`, `name: String`, `password: String`, `role: String`, `language: String`.
      - Validate: email contains `@` and `.`; name non-empty after trim; password non-empty;
        role ∈ `{ADMIN, MANAGER, USER}`; language ∈ `{en, ar}`.
        Invalid input → `VALIDATION_ERROR`.
      - Call `require_admin` → get `acting_user_id`. Session lock released.
      - Lock DB as mutable (`let mut db = state.db.lock()...`).
      - Start transaction (`let tx = db.transaction()?`).
      - Look up acting admin user by `acting_user_id` via `models::find_user_by_id` **inside
        the transaction** to get email for the audit event.
      - Call `models::insert_user` on the transaction → receive the newly inserted `User`.
      - Write `USER_CREATED` audit event on the transaction: acting admin ID/email,
        entity type `"user"`, target user ID from returned `User`, safe changes JSON
        with email and role.
      - Commit transaction.
      - Convert returned `User` to `UserResponse` and return.
- [x] T005 [US1] Add `pub mod users;` to `src-tauri/src/commands/mod.rs`
- [x] T006 [US1] Register `create_user` in `src-tauri/src/lib.rs` `generate_handler![]`

**Checkpoint**: `create_user` works — valid users created, duplicate email returns `CONFLICT`,
invalid input returns `VALIDATION_ERROR`, non-ADMIN returns `PERMISSION_DENIED`.

---

## Phase 3: User Story 2 — View Users (Priority: P1)

**Goal**: ADMIN lists users with optional filters and fetches a single user by ID.

- [x] T007 [US2] Implement `list_users` command in `src-tauri/src/commands/users.rs`:
      - Input: `role: Option<String>`, `is_active: Option<bool>`.
      - Call `require_admin`. Release session lock.
      - Validate role filter if provided: must be one of `ADMIN`, `MANAGER`, `USER`.
        Invalid filter → `VALIDATION_ERROR`.
      - Lock DB (immutable `let db = ...`).
      - Call `models::list_users(&db, role_filter, is_active)`.
      - Map results to `Vec<UserResponse>` (without `password_hash`).
      - Valid filters matching no users return an empty list (not an error).
- [x] T008 [US2] Implement `get_user` command in `src-tauri/src/commands/users.rs`:
      - Input: `id: String`.
      - Call `require_admin`. Release session lock.
      - Lock DB. Call `models::find_user_by_id`.
      - Map to `UserResponse` with `created_at`/`updated_at` from DB.
      - Missing user → `NOT_FOUND`.
- [x] T009 [US2] Register `list_users` and `get_user` in `src-tauri/src/lib.rs`

**Checkpoint**: `list_users` and `get_user` work — role filter, active filter, missing user handling.

---

## Phase 4: User Story 3 — Update User (Priority: P2)

**Goal**: ADMIN updates email, name, role, or language. Only provided fields change. `is_active`
is never modified. Last active ADMIN cannot be demoted. All optional fields being `None` →
`VALIDATION_ERROR`.

- [x] T010 [US3] Implement `update_user` command in `src-tauri/src/commands/users.rs`:
      - Input: `id: String`, `email: Option<String>`, `name: Option<String>`,
        `role: Option<String>`, `language: Option<String>`.
      - Call `require_admin` → get `acting_user_id`. Session lock released.
      - **If all four optional fields are `None`**: return `VALIDATION_ERROR`
        ("At least one field must be provided").
      - Validate each provided field: email format if Some, name non-empty if Some,
        role ∈ `{ADMIN, MANAGER, USER}` if Some, language ∈ `{en, ar}` if Some.
      - Lock DB as mutable. Look up target user (→ `NOT_FOUND` if missing).
      - **Last-admin demotion guard**: if `target.role == "ADMIN"` AND `target.is_active == true`
        AND `new_role.is_some()` AND `new_role != Some("ADMIN")`:
        call `count_active_admins`. If `count <= 1` → `BAD_REQUEST`.
      - Start transaction using the same mutable DB lock.
      - Look up acting admin user by `acting_user_id` via `models::find_user_by_id` **inside
        the transaction** to get email for audit.
      - Call `models::update_user` on the transaction → receive updated `User`.
      - Write `USER_UPDATED` audit: acting admin ID/email, target user ID, entity type `"user"`,
        changes JSON listing which fields were updated (use safe labels like
        `"email": "<updated>"`, `"role": "{new_role}"`; never include passwords or hashes).
      - Commit transaction.
      - Convert returned `User` to `UserResponse` and return.
- [x] T011 [US3] Register `update_user` in `src-tauri/src/lib.rs`

**Checkpoint**: Partial updates work, no-op update returns `VALIDATION_ERROR`, last-admin demotion
blocked, `is_active` unchanged.

---

## Phase 5: User Story 4 — Activate / Deactivate User (Priority: P2)

**Goal**: ADMIN toggles `is_active`. Deactivation revokes all refresh tokens in the same transaction.
Self-deactivation and last-admin deactivation are blocked.

- [x] T012 [US4] Implement `set_user_active` command in `src-tauri/src/commands/users.rs`:
      - Input: `id: String`, `is_active: bool`.
      - Call `require_admin` → get `acting_user_id`. Session lock released.
      - **Self-deactivation guard**: if `id == acting_user_id` AND `is_active == false` →
        `BAD_REQUEST`. (Activating self is a no-op, not an error.)
      - Lock DB as mutable (`let mut db = state.db.lock()...`).
      - Look up target user (→ `NOT_FOUND` if missing).
      - **Last-admin deactivation guard**: if `is_active == false` AND `target.role == "ADMIN"`
        AND `target.is_active == true`:
        call `count_active_admins`. If `count <= 1` → `BAD_REQUEST`.
      - **Start a transaction using this same mutable DB lock. Do not re-lock.**
      - Look up acting admin user by `acting_user_id` via `models::find_user_by_id` **inside
        the transaction** to get email for audit.
      - Call `models::set_user_active(&tx, &id, is_active)` → receive updated `User`.
      - If deactivating (`is_active == false`):
        - Call `models::revoke_all_refresh_tokens_for_user(&tx, &id)`.
        - Write `USER_DEACTIVATED` audit: acting admin ID/email, target user ID,
          entity type `"user"`, changes with target_id and target_role.
      - If activating (`is_active == true`):
        - Write `USER_ACTIVATED` audit with same structure.
      - Commit transaction.
      - Convert returned `User` to `UserResponse` and return.
- [x] T013 [US4] Register `set_user_active` in `src-tauri/src/lib.rs`

**Checkpoint**: Activation/deactivation works, self-deactivation blocked, last-admin deactivation
blocked, refresh tokens revoked on deactivation, correct audit events.

---

## Phase 6: Frontend Service Wrapper

**Purpose**: Typed invoke wrappers for the frontend. No UI pages.

- [x] T014 Create `src/services/users.ts` with typed service functions:
      - `createUser(email, name, password, role, language) -> Promise<UserResponse>`
      - `listUsers(role?, isActive?) -> Promise<UserResponse[]>`
      - `getUser(id) -> Promise<UserResponse>`
      - `updateUser(id, email?, name?, role?, language?) -> Promise<UserResponse>`
      - `setUserActive(id, isActive) -> Promise<UserResponse>`
      - Use existing `invokeCommand` from `src/lib/tauri.ts`. Never call `invoke()` directly.
      - Export `UserResponse` TypeScript interface matching the Rust DTO.
      - Do not create any UI pages or components.

---

## Phase 7: Polish & Verification

**Purpose**: Run approved checks, verify edge cases.

- [ ] T015 Verify that `CONFLICT`, `BAD_REQUEST`, and `UNAUTHORIZED` error codes exist in
      `src-tauri/src/error.rs`. If any is missing, add it consistently with the existing
      `AppError::new("CODE", "message")` convention. Keep database errors sanitized.
- [ ] T016 Run `cd src-tauri && cargo check` and fix all errors
- [ ] T017 Run `cd src-tauri && cargo clippy -- -D warnings` and fix all warnings
- [ ] T018 Run `npx tsc --noEmit` and fix all TypeScript errors
- [ ] T019 Run `npm run lint` and fix all lint errors
- [ ] T020 Manual verification — run the app and validate:
      - ADMIN creates a user → receives safe `UserResponse`
      - Duplicate email → `CONFLICT`
      - Non-ADMIN → `PERMISSION_DENIED`
      - No session → `UNAUTHORIZED`
      - List users with role/active filters
      - Get user by ID, missing user → `NOT_FOUND`
      - Update user partial fields; no-op → `VALIDATION_ERROR`
      - Demote last active ADMIN → `BAD_REQUEST`
      - Deactivate user → refresh tokens revoked
      - Self-deactivation → `BAD_REQUEST`
      - Activate/reactivate user → correct audit event
      - No `password_hash` in any response

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — can start immediately. BLOCKS all user stories.
- **User Stories (Phase 2-5)**: All depend on Phase 1 completion.
  - US1 (Create): T003 → T004 → T005 → T006 (sequential; T005/T006 depend on T003/T004).
    `commands/mod.rs` must expose `pub mod users;` (T005) before `lib.rs` registers
    `commands::users::*` (T006).
  - US2 (View): T007 → T008 → T009 (sequential; all edit `commands/users.rs` and `lib.rs`).
  - US3 (Update): T010 → T011.
  - US4 (Activate/Deactivate): T012 → T013.
- **Frontend (Phase 6)**: Depends on all commands registered (Phase 2-5).
- **Polish (Phase 7)**: Depends on all prior phases. T015 (error codes) must precede T016.

### Within Each Phase

- T001 and T002 can run in parallel (different files: `audit.rs`, `models/mod.rs`).
- T003 (require_admin helper + DTO) before T004 (create_user).
- T005 and T006 can be done together with T004.
- T007 and T008 are **sequential** (both modify `commands/users.rs`).
- T010, T012 have no intra-phase dependencies.
- T016 and T018 are independent checks but T015 must run first.

### Parallel Opportunities

```text
Phase 1: T001 ∥ T002
Phase 2: T003 → T004 + T005 + T006
Phase 3: T007 → T008 → T009
Phase 4: T010 → T011
Phase 5: T012 → T013
Phase 6: T014 (single file)
Phase 7: T015 → T016 → T017 → T018 → T019 → T020
```

**Registration order**: `commands/mod.rs` must be updated with `pub mod users;` (T005)
before `lib.rs` can register `commands::users::*` (T006). Same pattern applies to all
subsequent registration tasks. A command function must exist before it is registered in
`generate_handler![]`.

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Foundational (audit + models)
2. Complete Phase 2: User Story 1 (Create User)
3. Complete Phase 3: User Story 2 (View Users)
4. **STOP and VALIDATE**: Create a user and list/view it
5. Proceed to remaining user stories

### Incremental Delivery

1. Phase 1 → Foundation ready
2. Phase 2 + 3 (US1 + US2) → Create and view users
3. Phase 4 (US3) → Update users
4. Phase 5 (US4) → Activate/deactivate users
5. Phase 6 → Frontend wrappers
6. Phase 7 → Polish and final verification

---

## Notes

- [P] tasks = different files, no dependencies
- [P] does NOT apply to tasks editing the same file, even if they touch different functions
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Do not run `cargo test`, `npm test`, or any test command unless explicitly requested
- Use only project-approved checks: `cargo check`, `cargo clippy`, `npx tsc --noEmit`,
  `npm run lint`
- All commands MUST release `AppState.session` lock before acquiring `AppState.db` lock,
  to avoid holding both locks simultaneously
- Model mutation helpers (`insert_user`, `update_user`, `set_user_active`) return `User`
  so commands do not need to re-fetch after mutation
- `update_user` with all `None` optional fields returns `VALIDATION_ERROR` (no no-op updates)
- `list_users` with an invalid role filter returns `VALIDATION_ERROR`; valid filters
  matching no users return an empty list
- Last-admin demotion guard applies only when the target user is currently active ADMIN
  AND the requested new role is non-ADMIN
- Last-admin deactivation guard applies only when the target user is currently active ADMIN
  AND `is_active` is `false`
- Self-deactivation returns `BAD_REQUEST` regardless of admin count
- `seed.rs` is not modified; the debug-only `admin@daraerp.local` account remains available
