# Implementation Plan: User Management

**Branch**: `002-user-management` | **Date**: 2026-07-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/002-user-management/spec.md`

## Summary

Implement administrator-only user management commands for DaraERP: create, list, get, update, and
activate/deactivate user accounts. All commands require an active authenticated ADMIN session and are
guarded by the existing role-authorization helper. Mutations that combine user updates, token
revocation, and audit logging run inside a single database transaction. This feature replaces reliance
on the debug-only `seed.rs` fixture for normal account creation while keeping the seed available as a
development convenience. Scope is backend commands plus typed frontend service wrappers only; no user
management UI pages.

## Technical Context

**Language/Version**: Rust 1.75+ (backend), TypeScript 5.8+ (frontend)

**Primary Dependencies**:
- Backend: tauri 2, rusqlite (bundled), argon2, uuid, chrono, serde (all already installed by 001)
- Frontend: @tauri-apps/api, react 19 (already installed)

**No new dependencies required** — all crates are present from Feature 001.

**Storage**: SQLite (rusqlite bundled) for users, refresh tokens, and audit logs.

**Testing**: `cargo check` / `cargo clippy` for Rust; `npx tsc --noEmit` for TypeScript. No test
suite required per project rules.

**Target Platform**: Windows (.exe/.msi), macOS (.dmg), Linux (.deb/.AppImage)

**Project Type**: desktop-app (Tauri 2.5)

**Constraints**: Single-user local app; no HTTP server, no CORS, no multi-tenancy; CSP enforced;
least-privilege Tauri capabilities

**Scale/Scope**: 5 Tauri commands (create_user, list_users, get_user, update_user, set_user_active);
no new frontend pages

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Financial Precision | N/A | User Management does not handle monetary values |
| II. Security-First | PASS | ADMIN-only commands guarded by `auth::guard_role`; passwords hashed via argon2id via existing `auth::hash_password`; no secrets exposed in responses; refresh tokens revoked on deactivation |
| III. Local-First Desktop Architecture | PASS | All DB operations via rusqlite; no network calls; no HTTP endpoints |
| IV. Backend Authority | PASS | All business logic in `models/mod.rs` queries; thin commands in `commands/users.rs`; frontend invokes via service wrapper |
| V. Auditable Mutations | PASS | Explicit `audit_log()` calls for USER_CREATED, USER_UPDATED, USER_ACTIVATED, USER_DEACTIVATED inside transactions |
| Error Handling Convention | PASS | All commands return `Result<T, AppError>`; CONFLICT, VALIDATION_ERROR, BAD_REQUEST, PERMISSION_DENIED, UNAUTHORIZED, NOT_FOUND, DATABASE_ERROR codes used |
| Hard Constraints | PASS | No f64 money; no unwrap in commands; no plaintext secrets; no CSP null |

**Verdict**: All applicable gates PASS. No violations to track.

## Project Structure

### Documentation (this feature)

```text
specs/002-user-management/
├── spec.md              # Stakeholder-facing specification (this file)
├── plan.md              # This file: technical implementation plan
├── contracts/           # Tauri command signatures (optional)
└── tasks.md             # /speckit.tasks output
```

### Source Code

```text
src-tauri/src/
├── lib.rs                           # Updated: register user commands in generate_handler![]
├── error.rs                         # Existing: AppError (CONFLICT, BAD_REQUEST used via new())
├── audit.rs                         # Updated: +USER_CREATED, USER_UPDATED, USER_ACTIVATED,
│                                    #   USER_DEACTIVATED action constants
├── commands/
│   ├── mod.rs                       # Updated: +pub mod users
│   └── users.rs                     # NEW: create_user, list_users, get_user, update_user,
│                                    #   set_user_active commands + UserResponse DTO
└── models/
    └── mod.rs                       # Updated: +insert_user, list_users, update_user,
                                     #   set_user_active, count_active_admins,
                                     #   revoke_all_refresh_tokens_for_user query helpers

src/
└── services/
    └── users.ts                     # NEW: createUser(), listUsers(), getUser(), updateUser(),
                                     #   setUserActive() typed wrappers
```

## Complexity Tracking

No constitution violations — table intentionally empty.

### Implementation Order

1. **Audit action constants** — add USER_CREATED, USER_UPDATED, USER_ACTIVATED, USER_DEACTIVATED
   to `audit.rs`
2. **Model query helpers** — add user CRUD queries to `models/mod.rs`:
   - `insert_user`, `list_users`, `update_user`, `set_user_active`, `count_active_admins`,
     `revoke_all_refresh_tokens_for_user`
3. **Tauri commands** — create `commands/users.rs` with all 5 ADMIN-guarded commands
4. **Module registration** — add `pub mod users` to `commands/mod.rs` and register all 5 commands
   in `lib.rs`
5. **Frontend wrappers** — create `src/services/users.ts` with typed invoke functions

## Tasks Readiness Notes

Guidelines for `/speckit.tasks`:

- **Order**: Audit constants → model queries → commands → registration → frontend wrappers. Each
  layer depends on the previous.
- **Session and role guard**: Every command must check session existence (→ `UNAUTHORIZED`) and
  then ADMIN role (→ `PERMISSION_DENIED`). Use `require_admin` helper pattern.
- **Transaction wrapping**: Every mutation (create_user, update_user, set_user_active) must wrap
  DB change + audit inside `conn.transaction()`. `update_user` and `set_user_active` also check
  `count_active_admins` before demoting/deactivating. `set_user_active` (deactivate) also calls
  `revoke_all_refresh_tokens_for_user`.
- **Error handling**: Map SQLite constraint violations to `CONFLICT` at the model layer. Return
  `BAD_REQUEST` for self-deactivation and last-admin violations. Sanitize internal DB errors.
- **UserResponse DTO**: Defined in `commands/users.rs`. Never includes `password_hash`.
  Includes `id`, `email`, `name`, `role`, `language`, `is_active`, `created_at`, `updated_at`.
- **No test commands**: Do not run `cargo test`, `npm test`, or any test command unless explicitly
  requested. Use only the project-approved checks: `cargo check`, `cargo clippy`, `npx tsc --noEmit`,
  `npm run lint`.
- **seed.rs**: Left untouched. Still available as debug-only convenience.
- update_user with no provided fields must return VALIDATION_ERROR. Do not refresh updated_at or write audit when no meaningful change was requested.
- Invalid role filter returns VALIDATION_ERROR. Valid filters that match no users return an empty list.
- Commands should capture acting admin identity from AppState.session first, then release the session lock before opening a database transaction. Do not hold the session mutex while performing DB writes or audit logging.
- Model helpers used inside transactions must work with rusqlite::Transaction references. Use &tx / &*tx as needed, or structure helpers so they can be called consistently from both Connection and Transaction contexts.
- Last-admin demotion applies only when the target user is currently active ADMIN and the requested new role is non-ADMIN. Last-admin deactivation applies only when the target user is currently active ADMIN and is_active=false.