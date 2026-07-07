# Feature Specification: Foundation — Fix Critical Gaps Before Feature Work

**Feature Branch**: `000-foundation`

**Created**: 2026-07-07

**Status**: Draft

**Input**: Audit of DaraERP codebase revealed 5 critical and 8 high-severity foundation gaps that block all feature work per the constitution.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Financial Precision Fix (Priority: P1)

The SQLite schema currently stores 11 monetary columns as `REAL`, which violates the constitution's non-negotiable Financial Precision principle. All money MUST be stored as integer minor units (cents/piasters).

**Why this priority**: Constitution Principle I is non-negotiable. Every feature build on this schema would propagate floating-point money bugs.

**Independent Test**: Inspect `001_init.sql` — zero `REAL` columns remain on monetary fields. All money columns renamed to `*_minor` suffix with `INTEGER` type.

**Acceptance Scenarios**:

1. **Given** `001_init.sql` has `REAL` columns on `products.current_price`, `invoices.subtotal`, etc., **When** migration is fixed, **Then** all 11 money columns are `INTEGER` with `_minor` suffix, and `tax_rate` columns are `INTEGER` with `_bps` suffix.
2. **Given** updated schema, **When** `cargo check` runs, **Then** it compiles without reference to old column names.
3. **Given** a fresh init, **When** the DB is created, **Then** data can be inserted with integer minor-unit values.

---

### User Story 2 — Versioned Migration Runner (Priority: P1)

Current `db.rs` uses `include_str!("...001_init.sql")` + `execute_batch()` on every startup — the constitution explicitly forbids this as a long-term strategy.

**Why this priority**: Without a migration runner, Phase 1 schema changes (002+, 003+, etc.) cannot be applied. The `include_str!` anti-pattern must be replaced before any schema work.

**Independent Test**: Delete the SQLite DB file. Start the app. A `schema_migrations` table exists, `001_init.sql` is recorded as applied, and a second startup does not re-run the migration. Adding a `002_test.sql` and restarting applies it and records it.

**Acceptance Scenarios**:

1. **Given** no SQLite DB, **When** app starts, **Then** `schema_migrations` table is created, `001_init.sql` runs inside a transaction, and version `001` is recorded with timestamp.
2. **Given** `001_init.sql` already applied, **When** app starts, **Then** migration is skipped (idempotent).
3. **Given** a new `migrations/002_*.sql` file, **When** app starts, **Then** only `002` runs in its own transaction and is recorded.
4. **Given** a migration that fails midway, **When** app starts, **Then** the transaction rolls back, the migration is NOT recorded, and startup fails with a clear `AppError`.
5. **Given** the migration runner is active, **When** `include_str!` + `execute_batch` pattern no longer exists in `db.rs`, **Then** the constitution violation is resolved.

---

### User Story 3 — Error Handling Convention (Priority: P1)

`AppError` exists but has gaps: missing `std::error::Error` impl, inconsistent error codes, and raw SQLite error messages are exposed to the frontend. `db::init_db` returns `Box<dyn std::error::Error>` instead of `AppError`.

**Why this priority**: Constitution mandates a single error type convention. Commands cannot be implemented without consistent error handling. Every command must return `Result<T, AppError>`.

**Independent Test**: Compile a command that returns `Result<(), AppError>`. Use `?` on a `rusqlite` operation — it converts to `AppError` via `From`. Frontend receives clean error codes like `NOT_FOUND`, not raw SQLite strings.

**Acceptance Scenarios**:

1. **Given** `AppError` in `error.rs`, **When** `impl std::error::Error for AppError` is added, **Then** it can be used in `Box<dyn Error>` and `anyhow` contexts.
2. **Given** `From<rusqlite::Error>`, **When** a constraint violation occurs, **Then** the frontend receives `DB_CONSTRAINT` with a sanitized message, not raw SQLite error text.
3. **Given** error codes, **When** reviewed against the convention, **Then** they follow the `UPPER_SNAKE_CASE` pattern: `DATABASE_ERROR`, `VALIDATION_ERROR`, `NOT_FOUND`, `AUTH_INVALID_CREDENTIALS`, `PERMISSION_DENIED`, `INTERNAL_ERROR`.
4. **Given** `db::init_db`, **When** the function signature is updated, **Then** it returns `Result<Connection, AppError>` not `Box<dyn std::error::Error>`.

---

### User Story 4 — Lint and Quality Scripts (Priority: P2)

`package.json` has no `lint` script. No ESLint or Prettier is configured. No `cargo clippy` enforcement in `Cargo.toml`. Constitution requires build/quality commands and AGENTS.md references `npm run lint` as a documented command.

**Why this priority**: Quality gates cannot be enforced without tooling. Constitution compliance checks require lint/format steps.

**Independent Test**: Run `npm run lint` — ESLint runs without errors. Run `cargo clippy -- -D warnings` — passes with zero warnings.

**Acceptance Scenarios**:

1. **Given** `package.json`, **When** `"lint": "eslint ."` is added, **Then** `npm run lint` executes successfully.
2. **Given** the project, **When** ESLint and Prettier are installed and configured with sensible defaults, **Then** `npx eslint .` and `npx prettier --check .` pass.
3. **Given** `Cargo.toml`, **When** `[lints.clippy]` section is added with deny-by-default rules, **Then** `cargo clippy -- -D warnings` catches issues.
4. **Given** `.clippy.toml` or `Cargo.toml` lints section, **When** a new Rust file is added, **Then** CI/cargo check enforces lint rules.

---

### User Story 5 — CSP and Security Baseline (Priority: P2)

`tauri.conf.json` has `"csp": null` — no Content Security Policy. Constitution requires a restrictive CSP before Phase 1.

**Why this priority**: Opening feature work without CSP is a security gap. The constitution mandates this before Phase 1.

**Independent Test**: Inspect `tauri.conf.json` — CSP is not `null`. App launches and functions with CSP enabled.

**Acceptance Scenarios**:

1. **Given** `tauri.conf.json`, **When** CSP is set to a restrictive default, **Then** it is not `null`.
2. **Given** the CSP, **When** the app runs in dev mode, **Then** hot-reload (HMR via WebSocket) still functions.
3. **Given** the CSP, **When** a `tauri://localhost` or `asset://localhost` resource is loaded, **Then** it is allowed.
4. **Given** the CSP, **When** an external `https://` script is loaded, **Then** it is blocked (unless explicitly allowed).

---

### User Story 6 — Typed Invoke Wrapper and Service Layer (Priority: P2)

`src/App.tsx` calls `invoke("greet", ...)` directly from a component. Constitution: "Components MUST NOT call `invoke()` directly. Use a typed wrapper service layer." No `src/lib/tauri.ts` or `src/services/` exists.

**Why this priority**: Establishes the frontend pattern that ALL features will follow. Without it, every feature would embed raw invoke calls in components.

**Independent Test**: Create a typed invoke wrapper. Call it from a component. The wrapper compiles and passes type-checking for both request args and response shape.

**Acceptance Scenarios**:

1. **Given** `src/lib/tauri.ts`, **When** a typed `invokeCommand<TResponse, TArgs>` function is defined, **Then** it wraps `@tauri-apps/api/core` invoke with TypeScript generics.
2. **Given** `src/App.tsx`, **When** the `greet` call is refactored, **Then** it uses the wrapper, not a direct `invoke()` import.
3. **Given** the wrapper, **When** a new command is added on the backend, **Then** a corresponding typed wrapper can be called from any service without repeating `invoke()` imports.

---

### Edge Cases

- What happens when SQLite DB already exists with REAL columns before the financial precision fix?
  → The app is in scaffold state. Drop the DB, re-run the migration. No migration path needed for existing data.
- What happens when a migration file has a syntax error?
  → Transaction rolls back. Migration NOT recorded in `schema_migrations`. App startup fails with a clear error message including the migration filename and the SQL error (sanitized).
- What happens if the CSP blocks Tauri's own IPC or HMR?
  → Must test in `npm run tauri dev`. HMR connects via WebSocket to `ws://localhost:1430` — this must be allowed.
- What about the `greet` command in `App.tsx` that has no backend handler?
  → Either register a minimal `greet` command on the backend OR remove the greet demo from `App.tsx` (replaced with a placeholder).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST store all monetary values as `INTEGER` minor units with `_minor` suffix column names in SQLite schema.
- **FR-002**: System MUST store tax rates as `INTEGER` basis points with `_bps` suffix (e.g., 1500 = 15.00%).
- **FR-003**: System MUST have a `schema_migrations` table tracking `version TEXT PRIMARY KEY`, `applied_at TEXT NOT NULL`.
- **FR-004**: System MUST discover and apply unapplied migrations in version order on startup.
- **FR-005**: System MUST wrap each migration in a transaction and record it only on success.
- **FR-006**: System MUST fail startup with a clear error if a migration fails.
- **FR-007**: System MUST NOT use `include_str!` + `execute_batch()` on every startup.
- **FR-008**: `AppError` MUST implement `std::error::Error`.
- **FR-009**: `AppError` error codes MUST use stable `UPPER_SNAKE_CASE` strings matching the documented convention.
- **FR-010**: `From<rusqlio::Error>` MUST sanitize error messages — never expose raw SQLite internals.
- **FR-011**: All Tauri commands and `db.rs` MUST return `Result<T, AppError>`, not `Box<dyn Error>`.
- **FR-012**: `package.json` MUST include a `"lint": "eslint ."` script.
- **FR-013**: ESLint and Prettier MUST be installed and configured with sensible TypeScript/React defaults.
- **FR-014**: `Cargo.toml` MUST include `[lints.clippy]` deny-by-default configuration.
- **FR-015**: `tauri.conf.json` `security.csp` MUST NOT be `null`.
- **FR-016**: CSP MUST allow Tauri IPC (`tauri://localhost`), assets, and dev-mode HMR.
- **FR-017**: `src/lib/tauri.ts` MUST export a typed `invokeCommand<TResponse, TArgs>` wrapper.
- **FR-018**: React components MUST NOT import `invoke` from `@tauri-apps/api/core` directly.

### Key Entities

- **Migration**: `version` (string, e.g. "001_init"), `applied_at` (ISO-8601 UTC timestamp). Stored in `schema_migrations`.
- **AppError**: `code` (stable uppercase string), `message` (human-readable, sanitized). Implements `std::error::Error`.
- **InvokeWrapper**: Generic TypeScript function `<TResponse, TArgs extends Record<string, unknown>>(cmd: string, args?: TArgs) => Promise<TResponse>`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero `REAL`, `FLOAT`, `DOUBLE`, `f64`, or `f32` types appear in any persisted money column or Rust model.
- **SC-002**: App starts cleanly on a fresh machine. Second startup is a no-op for migrations.
- **SC-003**: Adding a new SQL migration file and restarting applies it automatically.
- **SC-004**: `npm run lint` completes without errors.
- **SC-005**: `cargo clippy --all-targets --all-features -- -D warnings` passes with zero warnings.
- **SC-006**: `cargo check` compiles successfully with `AppError` as the canonical command return type.
- **SC-007**: CSP is not null and the app functions in both dev and build modes.
- **SC-008**: No React component imports `invoke` directly from `@tauri-apps/api/core`.

## Assumptions

- The existing SQLite DB is disposable (scaffold state) — no data migration needed for the REAL→INTEGER fix.
- Tax rates stored as basis points (bps) are acceptable per the README data model (no stakeholder objection).
- We are NOT adding all planned dependencies in this phase — only what's needed for the 5 critical + high gaps.
  Added dependencies: `eslint`, `prettier`, `eslint-plugin-react`, `@typescript-eslint/*` (dev only).
  NOT adding: `argon2`, `jsonwebtoken`, `tailwindcss`, etc. — those belong in their feature phases.
- The `greet` demo command from the Tauri scaffold is NOT needed for foundation. Either register a minimal command or remove the demo UI.
- Migration runner uses `rusqlite::Transaction` — no need for a migration framework crate.
- Dev mode HMR WebSocket at `ws://localhost:1430` is the default Vite port.
