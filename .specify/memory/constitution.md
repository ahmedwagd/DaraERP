<!--
  Sync Impact Report
  ==================
  Version change: 1.0.0 → 1.1.0
  Rationale: MINOR bump — materially expanded guidance (new Error Handling section,
    expanded Security/Frontend/Concurrency rules, added Hard Constraints section,
    enhanced all five core principles with concrete MUST/SHOULD rules)
  Modified principles:
    - I. Financial Precision: added concrete field naming convention, `rust_decimal` option
    - II. Security-First: added JWT_SECRET management, capabilities, CSP, secure-storage clarification
    - III. Local-First Desktop Architecture: added concurrency rules, PDF/Arabic requirements
    - IV. Backend Authority: added command registration rules, error handling, validation rules
    - V. Auditable Mutations: added audit entry schema fields
  Added sections:
    - Error Handling Convention (AppError, error codes, no unwrap/expect)
    - Hard Constraints (Do Not list)
  Removed sections: None
  Templates requiring updates:
    - .specify/templates/plan-template.md ✅ no changes needed (generic, filled at runtime)
    - .specify/templates/spec-template.md ✅ no changes needed (generic, filled at runtime)
    - .specify/templates/tasks-template.md ✅ no changes needed (generic, filled at runtime)
    - .specify/templates/checklist-template.md ✅ no changes needed (generic, filled at runtime)
  Follow-up TODOs: None
-->

# DaraERP Constitution

## Core Principles

### I. Financial Precision (Non-Negotiable)

All monetary values MUST be stored and computed as integer minor units (cents/piasters).
Use names that make the unit obvious: `current_price_minor`, `subtotal_minor`,
`tax_total_minor`, `grand_total_minor`, `unit_price_minor`, `line_total_minor`,
`discount_minor`, `price_minor`.

Floating-point types (`f32`, `f64`, `REAL`, `FLOAT`, `DOUBLE`) MUST NOT appear in any
persisted financial column. If decimal calculations are needed before persisting, use
`rust_decimal`; always persist as integer minor units.

Invoice totals, taxes, discounts, payments, balances, and price history calculations
MUST be deterministic and rounding-safe.

Price changes MUST be atomic within a single database transaction:
1. INSERT into `price_history`
2. UPDATE product `current_price_minor`
3. Emit audit log entry
4. Commit all together. If any step fails, roll back the full transaction.

Convert to display format only at the UI boundary. Never expose minor-unit arithmetic
to the frontend.

### II. Security-First

Authentication MUST use argon2 for password hashing with per-user salt. Sessions MUST
use JWT tokens with refresh token rotation and theft detection.

Token storage rules:
- Tokens MUST NOT be stored in `localStorage` or plaintext files.
- `tauri-plugin-store` is a persistent key-value store, NOT secure credential storage.
- If persistent secret storage is required, use an OS keychain/keyring approach.

`JWT_SECRET` MUST be generated, loaded, rotated, and protected — document how at
implementation time. Never hard-code secrets.

Role-based access control MUST be enforced via a `guard_role(user_role, &["ADMIN"])`
helper in Rust on every protected command. Never trust frontend-only permission checks.

The Tauri Content Security Policy MUST be restrictive (not `csp: null`). Tauri plugin
permissions and capabilities MUST follow least-privilege. Do not enable broad
filesystem or shell access.

Validate all file paths selected through dialogs. Treat imported files as untrusted
input. Do not log passwords, tokens, or filesystem secrets.

### III. Local-First Desktop Architecture

DaraERP is a single-user local desktop application. There is NO HTTP server, NO CORS,
NO rate-limiting, and NO multi-tenancy. SQLite runs embedded via `rusqlite` (bundled
feature) with no external database installation required.

PDF files MUST be saved to disk and opened with the system viewer, not served via HTTP.
File uploads MUST use Tauri native file dialog, not multer. Build outputs target:
`.exe`/`.msi` (Windows), `.dmg` (macOS), `.deb`/`.AppImage` (Linux).

Invoice PDFs are a core feature and MUST support Arabic correctly. Confirm the
approved PDF strategy before implementation: HTML/CSS-to-PDF via WebView pipeline,
Rust PDF generation with Arabic shaping+RTL support, or another documented approach.
Embed required fonts. Verify RTL layout, Arabic shaping, numbers, currency, and mixed
Arabic/English text.

Concurrency: `Mutex<rusqlite::Connection>` is acceptable for early single-user desktop
development. Do not hold the DB mutex while performing slow non-DB work. Keep
transactions short. Move heavy DB work to `spawn_blocking` or introduce a connection
pool (e.g., `deadpool-sqlite`) as the app grows.

### IV. Backend Authority (Business Logic in Rust)

All business logic, validation, and data mutations MUST reside in Rust backend modules.
The frontend MUST focus on interaction, display, validation feedback, and user
workflow.

Every former REST endpoint becomes a Tauri `#[command]`. Frontend `fetch()` calls
become typed `invoke()` calls through a service layer (`src/services/` or
`src/lib/tauri.ts`). Components MUST NOT call `invoke()` directly.

Command rules:
- Register every command in `tauri::generate_handler![]`.
- Keep command names stable — the frontend depends on them.
- Use typed request/response structs for complex inputs.
- Validate all command inputs before mutation.
- Never trust frontend-only validation.
- Commands that change data MUST: validate input → check permissions → run the DB
  transaction → write audit log → return typed response or stable error.

Commands MUST be thin. Put business logic in modules/services, not in the command
function body. Database mutations that belong together MUST run in one transaction.

### V. Auditable Mutations

Every tracked data mutation MUST emit an audit log entry via an explicit `audit_log()`
call in Rust. Audit records MUST capture:
- actor: user ID
- action: stable code (CREATE, UPDATE, DELETE, LOGIN, LOGOUT, etc.)
- entity: type (Customer, Product, Invoice, etc.) and ID
- before/after: snapshot of changed values where appropriate
- timestamp: UTC
- summary: human-readable description where useful

The audit log MUST be queryable and viewable through the application UI. All CRUD
operations on customers, products, invoices, and settings are auditable. Do NOT audit
sensitive secrets such as passwords or access tokens.

## Error Handling Convention

Use one backend error convention consistently:

1. Define `AppError` in `src-tauri/src/error.rs`.
2. Every command returns `Result<T, AppError>`.
3. Error codes are stable uppercase strings:
   - `AUTH_INVALID_CREDENTIALS`
   - `AUTH_EXPIRED_TOKEN`
   - `PERMISSION_DENIED`
   - `DB_CONSTRAINT`
   - `VALIDATION_ERROR`
   - `NOT_FOUND`
4. Never expose raw SQLite errors, filesystem paths, secrets, or stack traces to the
   frontend.
5. Frontend maps error codes to i18n messages.
6. Use `?` for propagation. Never use `unwrap()` or `expect()` inside commands.

## Technology Stack & Constraints

**Runtime**: Tauri v2, Rust, SQLite (rusqlite bundled), React 19, TypeScript, Vite

**Frontend libraries** (use existing, no new UI libs):
- shadcn/ui and Radix primitives, Tailwind CSS, lucide-react, recharts,
  react-i18next, Sonner

**Rust crates** (check before adding duplicates):
- rusqlite (bundled), jsonwebtoken, argon2, uuid, chrono, serde/serde_json,
  sha2, rand, printpdf

**Tauri plugins**: @tauri-apps/api, @tauri-apps/plugin-store, @tauri-apps/plugin-dialog,
@tauri-apps/plugin-fs

**Dependency policy**:
- Add dependencies only when the feature that requires them is being implemented.
- Before adding, check whether the dependency already exists.
- Prefer established, maintained packages.
- Document why a new dependency is needed if the purpose is not obvious.

**Constraints**:
- Do NOT add new UI libraries without justification.
- Do NOT add Rust crates that duplicate existing functionality.
- Do NOT expand features before core architectural decisions are resolved.

## Development Workflow

**Feature order**: 001→Auth, 002→Seed, 003→Invoices, 004→Customers,
005→Catalog+PriceHistory, 006→N/A, 007→Permissions, 008→N/A, 009→Fixes,
010→Notifications, 011→Audit, 012→Enhancements

Foundation work before feature work:
1. Fix documentation references.
2. Fix financial precision in schema and models.
3. Add a versioned migration runner.
4. Add missing lint/build scripts.
5. Establish security, error, and invoke conventions.

**First vertical slice**: Login → Protected Layout → Customers CRUD → Audit Entry →
SQLite Persistence.

**File conventions**:
- Rust source: `src-tauri/src/`
- Tauri commands: `src-tauri/src/commands/` (one file per module)
- Models: `src-tauri/src/models/` (Rust structs + DB query helpers)
- Migrations: `src-tauri/migrations/` (numbered SQL files)
- Frontend: `src/` (mirrors original client/src/ structure)

**Database migrations** MUST use a versioned migration runner:
- Migrations live in `src-tauri/migrations/` with numbered names
  (e.g., `001_init.sql`, `002_add_price_history_indexes.sql`).
- SQLite has a `schema_migrations` table.
- Startup applies only unapplied migrations.
- Each migration is applied inside a transaction.
- A failed migration MUST stop startup with a clear error.
- Never edit an already-applied migration (unless still in disposable local scaffold
  state).
- Do not use `include_str!("...001_init.sql")` + `execute_batch()` on every startup
  as the long-term strategy.

**Build/quality commands** (run from correct working directory):
```bash
npm run tauri dev       # Dev with hot reload
npm run tauri build     # Production build
npm run lint            # Frontend lint
cd src-tauri && cargo check
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
```

Do not run test commands unless explicitly asked (`npm test`, `cargo test`).

**When blocked**: Check `README.md` (architecture, data model, quick reference, feature
parity map). Check current code for actual implementation. Check legacy `specs/`,
`api/src/`, `client/src/` only if those paths exist. If docs and code conflict,
document the mismatch and make the smallest safe fix.

## Hard Constraints (Do Not)

1. Do not use `f64`, `f32`, `REAL`, `FLOAT`, or `DOUBLE` for persisted money.
2. Do not use `unwrap()` or `expect()` in Tauri commands.
3. Do not store secrets in plaintext.
4. Do not store tokens in `localStorage`.
5. Do not treat `tauri-plugin-store` as secure secret storage.
6. Do not skip migrations or mutate schema ad hoc.
7. Do not call `invoke()` directly from React components.
8. Do not put business logic inside UI components.
9. Do not rely on frontend-only permission checks.
10. Do not add broad filesystem or shell permissions.
11. Do not leave `"csp": null` for production.
12. Do not add new UI libraries without approval.
13. Do not implement PDF generation without confirming Arabic/RTL support.
14. Do not run tests unless explicitly asked.
15. Do not expose raw database errors, file paths, secrets, or stack traces to the
    frontend.

## Governance

This constitution supersedes all other development practices for DaraERP. All
implementation decisions and spec-kit plans MUST comply with these principles.

**Amendment procedure**: Proposed changes MUST be documented with rationale, reviewed
against existing principles for conflicts, and version-bumped according to semantic
versioning:
- MAJOR: Principle removal or redefinition.
- MINOR: New principle or section added, or materially expanded guidance.
- PATCH: Clarifications, wording, typo fixes.

**Compliance**: Every `/speckit.plan` MUST include a Constitution Check gate. Every
PR/commit that violates a principle MUST include explicit justification in the plan's
Complexity Tracking table.

**Version**: 1.1.0 | **Ratified**: 2026-07-07 | **Last Amended**: 2026-07-07
