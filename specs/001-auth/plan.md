# Implementation Plan: Authentication & Session Management

**Branch**: `001-auth` | **Date**: 2026-07-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-auth/spec.md`

## Summary

Implement local desktop authentication for DaraERP: secure password verification, short-lived access
credentials with rotating renewal credentials, theft detection via credential family tracking,
approved secure platform credential storage, role-based access control (ADMIN/MANAGER/USER), audit
logging for security events, i18n-enabled login screen (Arabic/English), and silent session restore
on app restart.

Feature 001 implements the authentication and session foundation plus a reusable role-authorization
helper. Only auth/session-related Tauri commands are registered. Full RBAC enforcement on invoice,
customer, product, and settings commands will be applied when those features are implemented later.
No placeholder business commands are created just to test RBAC.

## Technical Context

**Language/Version**: Rust 1.75+ (backend), TypeScript 5.8+ (frontend)

**Primary Dependencies**:
- Backend: tauri 2, rusqlite (bundled), argon2, jsonwebtoken, sha2, rand, uuid, chrono, serde, keyring
- Frontend: @tauri-apps/api, react 19, react-router, react-i18next, i18next

**Storage**: SQLite (rusqlite bundled) for session records and audit events; OS keychain (keyring crate) for JWT secret and refresh tokens

**Testing**: `cargo check` / `cargo clippy` for Rust; `npx tsc --noEmit` for TypeScript. No test suite required in v1 per project rules.

**Target Platform**: Windows (.exe/.msi), macOS (.dmg), Linux (.deb/.AppImage)

**Project Type**: desktop-app (Tauri 2.5)

**Performance Goals**: login < 3s (including argon2), silent restore < 2s, error display < 1s

**Constraints**: Single-user local app; no HTTP server, no CORS, no multi-tenancy; CSP enforced; least-privilege Tauri capabilities

**Scale/Scope**: 1–10 local user accounts; 3 roles; 4 Tauri commands (login, logout, refresh_session, get_current_user); 3 frontend pages (Login, Dashboard, Layout)

**Initial Admin Account Prerequisite**: Feature 001 depends on at least one active ADMIN account
existing in the database before login can be validated end-to-end. This account may be provided by
foundation setup, a development-only seed helper, direct SQL during local development, or a test
fixture. This is not a public registration flow, and this prerequisite must not expand Feature 001
into full user management. `/speckit.tasks` must include a small prerequisite task for providing a
temporary local development admin account for validation.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Financial Precision | N/A | Auth does not handle monetary values |
| II. Security-First | PASS | Approved password hashing; short-lived + rotating renewal credentials + theft detection; OS keychain (not store); centralized role check for all privileged commands; restrictive CSP already applied |
| III. Local-First Desktop Architecture | PASS | No HTTP server; SQLite embedded; PDF not involved in this feature |
| IV. Backend Authority | PASS | All auth logic in Rust `auth.rs`; thin commands in `commands/auth.rs`; frontend invokes via wrapper |
| V. Auditable Mutations | PASS | Explicit audit_log() calls for login, logout, failed login, theft detection |
| Error Handling Convention | PASS | All commands return `Result<T, AppError>`; stable error codes used |
| Hard Constraints | PASS | No f64 money; no unwrap in commands; no localStorage; no plaintext secrets; no CSP null |

**Pre-Phase 0 verdict**: All applicable gates PASS. No violations to track.

**Post-Phase 1 re-check**: All applicable gates still PASS. No violations to track.

## Project Structure

### Documentation (this feature)

```text
specs/001-auth/
├── plan.md              # This file
├── research.md          # Phase 0: technology decisions
├── data-model.md        # Phase 1: database entities
├── quickstart.md        # Phase 1: validation guide
├── contracts/           # Phase 1: Tauri command signatures
│   └── auth-commands.md
└── tasks.md             # Phase 2: /speckit.tasks output
```

### Source Code

```text
src-tauri/src/
├── lib.rs                          # Updated: register auth commands + load JWT secret
├── db.rs                           # Existing: init_db + run_migrations
├── error.rs                        # Existing: AppError
├── auth/
│   └── mod.rs                      # NEW: hash_password, verify_password, issue_tokens,
│                                   #   verify_token, rotate_refresh_token, detect_theft,
│                                   #   revoke_family, load_jwt_secret
├── commands/
│   ├── mod.rs                      # Updated: add auth commands
│   └── auth.rs                     # NEW: login, logout, refresh_session, get_current_user
└── models/
    └── mod.rs                      # Updated: User model + DB queries

src/
├── lib/
│   └── tauri.ts                    # Existing: invokeCommand wrapper
├── services/
│   └── auth.ts                     # NEW: login(), logout(), refreshSession(), getCurrentUser()
├── contexts/
│   └── AuthContext.tsx             # NEW: React context for auth state
├── components/
│   ├── ProtectedRoute.tsx          # NEW: route guard
│   └── Layout.tsx                  # NEW: sidebar + header with role-based nav
├── pages/
│   ├── Login.tsx                   # NEW: login screen with i18n
│   └── Dashboard.tsx               # NEW: placeholder protected page
├── i18n/
│   ├── index.ts                    # NEW: i18n setup
│   ├── en.json                     # NEW: English strings
│   └── ar.json                     # NEW: Arabic strings
├── App.tsx                         # Updated: router + AuthProvider
└── main.tsx                        # Unchanged
```

**Structure Decision**: Desktop app with frontend-backend split. Backend auth logic isolated in `auth/mod.rs`; commands thin wrappers in `commands/auth.rs`. Frontend follows React context + service layer pattern. No test directories per project rules (no tests unless asked).

## Complexity Tracking

> No constitution violations — table intentionally empty.

## Tasks Readiness Notes

Guidelines for `/speckit.tasks`:

- **Initial admin prerequisite**: Include a prerequisite task for providing a development-only ADMIN
  user before login can be validated. Options: direct SQL insert, a `seed.rs` helper called once, or
  a Tauri command that creates the account only when no users exist. Do not build full user
  management.
- **Order**: Reusable auth/session modules (`auth/mod.rs`) before command wiring (`commands/auth.rs`).
  Secure storage and session restore before protected layout validation.
- **Role authorization**: Include a reusable role-authorization helper (`guard_role()`). Apply it to
  auth/session operations where relevant. Document required role arrays for future feature commands.
  Do not implement non-auth business commands.
- **Logout workflow**: Revoke active renewal credentials, clear local secure session state, clear
  in-memory session, return user to login screen. If any step fails, report a safe error and do not
  leave the UI showing an active logged-out state while a usable local session remains. This is a
  safe logout workflow, not a single atomic transaction across storage systems.
- **Validation**: Include quickstart validation as final manual verification.
- **No test commands**: Do not run `cargo test`, `npm test`, or any test command unless explicitly
  requested. Use only the project-approved checks: `cargo check`, `cargo clippy`, `npx tsc --noEmit`,
  `npm run lint`.
