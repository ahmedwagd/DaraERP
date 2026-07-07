# AGENTS.md — DaraERP Tauri Edition

Project: DaraERP Tauri Edition — desktop business management system  
Tech: React 19 + Vite + TypeScript frontend, Rust + Tauri 2 + SQLite backend  
Blueprint: Read `README.md` first for architecture, data model, migration strategy, command signatures, and feature parity mapping.

> This file is project context for coding agents. Keep tool-specific behavior, tone preferences, and assistant style rules outside this file.

---

## Operating Rules

1. Do not run `cargo test`, `npm test`, or other test commands unless explicitly asked.
2. Before implementation, read `README.md`, especially:
   - Current Implementation State
   - Non-Negotiable Architecture Decisions
   - Data Model
   - Quick Reference
   - Feature Parity Map
   - Roadmap
3. If `README.md` and code disagree, treat the code as the current implementation and the README as the intended target. Note the mismatch before changing code.
4. Prefer small, reviewable changes over broad rewrites.
5. Do not add dependencies unless they are needed for the current task or listed in the relevant project phase.

---

## Spec Kit

Spec Kit (`github/spec-kit`) is installed. Slash commands available:

- `/speckit.constitution` — project principles
- `/speckit.specify` — define requirements and user stories
- `/speckit.clarify` — clarify underspecified areas
- `/speckit.plan` — technical implementation plan
- `/speckit.tasks` — actionable task breakdown
- `/speckit.analyze` — cross-artifact consistency check
- `/speckit.checklist` — quality checklist generation
- `/speckit.implement` — execute tasks per plan
- `/speckit.converge` — assess and append remaining work

Use Spec Kit for new feature planning, large refactors, and cross-document consistency checks.

---

## Feature Order

Follow the phase order defined in `README.md` Roadmap section:

1. **Phase 0**: Foundation and consistency (schema fixes, migration runner, scripts, conventions)
2. **Phase 1**: Minimal working vertical slice (auth, seed, customers CRUD, audit)
3. **Phase 2**: Invoicing core (products, categories, invoices, PDF)
4. **Phase 3**: Supporting ERP features (notifications, audit viewer, permissions, reports)

Foundation work comes before feature work. Do not skip phases.

---

## File Conventions

- Rust source: `src-tauri/src/`
- Tauri commands: `src-tauri/src/commands/`
- Rust models and DB helpers: `src-tauri/src/models/`
- Migrations: `src-tauri/migrations/`
- Frontend source: `src/`
- Shared frontend utilities: `src/lib/`
- Frontend services/invoke wrappers: `src/services/` or `src/lib/tauri.ts`
- Frontend features/pages: `src/features/<feature>/`
- Tauri config: `src-tauri/tauri.conf.json`
- Rust dependencies: `src-tauri/Cargo.toml`
- Frontend dependencies/scripts: `package.json`

---

## Build and Quality Commands

Run commands from the correct working directory.

```bash
# Repository root
npm run tauri dev
npm run tauri build
npm run lint

# Rust backend
cd src-tauri && cargo check
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
```

Do not run test commands unless explicitly asked:

```bash
npm test
npm run test
cargo test
```

If `npm run lint` is documented but missing from `package.json`, add the script before relying on it.

---

## Core Architecture Rules

1. Every former REST endpoint becomes a Tauri `#[command]`.
2. Frontend `fetch()` calls become typed `invoke()` calls.
3. Components must not call `invoke()` directly. Use a service layer or `src/lib/tauri.ts` wrapper.
4. SQLite schema should mirror the Prisma business model, but use desktop-appropriate storage and migration patterns.
5. All command inputs and outputs must be serializable with `serde`.
6. Commands should be thin. Put business logic in modules/services, not directly in the command function.
7. Database mutations that belong together must run in one transaction.
8. Audit logging must happen after every tracked mutation.
9. Role checks must happen before privileged mutations.
10. User-facing strings in the frontend should use i18n keys, not hard-coded English or Arabic strings.

---

## Financial Precision Rules

This is a hard rule.

1. Do not persist money as `REAL`, `FLOAT`, `DOUBLE`, `f32`, or `f64`.
2. Store monetary values as integer minor units, such as cents or piasters.
3. Use names that make the unit obvious:
   - `current_price_minor`
   - `subtotal_minor`
   - `tax_total_minor`
   - `grand_total_minor`
   - `unit_price_minor`
   - `line_total_minor`
   - `discount_minor`
   - `price_minor`
4. Convert to display format only at the UI boundary.
5. If decimal calculations are needed, use a decimal-safe strategy such as `rust_decimal`, then persist integer minor units.
6. Invoice totals, taxes, discounts, payments, balances, and price history must never use floating-point storage.

If `001_init.sql` contains `REAL` money columns, fix the schema before feature work.

---

## Migration Rules

The app must use versioned migrations.

Required behavior:

1. Migrations live in `src-tauri/migrations/`.
2. Migration files use numbered names, for example:
   - `001_init.sql`
   - `002_add_price_history_indexes.sql`
   - `003_add_invoice_status.sql`
3. SQLite has a `schema_migrations` table.
4. Startup applies only migrations that have not yet been applied.
5. Each migration is applied inside a transaction.
6. A failed migration must stop startup and return a clear error.
7. Do not run `include_str!("...001_init.sql")` + `execute_batch()` on every startup as the long-term strategy.
8. Never edit an already-applied migration unless the project is still in disposable local scaffold state.

---

## Database and Concurrency Rules

Current acceptable scaffold pattern:

- `Mutex<rusqlite::Connection>` is acceptable for early single-user desktop development.

Required caution:

1. Do not hold the DB mutex while performing slow non-DB work.
2. Keep transactions short.
3. Do not perform long synchronous DB operations directly in async UI-facing paths if they can block responsiveness.
4. As the app grows, move heavy DB work to `spawn_blocking` or introduce a connection pool such as `deadpool-sqlite`.
5. Document any command expected to perform large reads, PDF generation, imports, exports, or batch updates.

---

## Auth and Session Rules

Planned auth stack:

- Password hashing: `argon2`
- JWT handling, if JWT remains part of the local architecture: `jsonwebtoken`
- Token hashing: `sha2`
- Random token generation: `rand`

Rules:

1. Hash passwords with Argon2. Never store plaintext passwords.
2. Prefer short-lived access tokens or local session state.
3. Refresh token rotation must detect reuse/theft if refresh tokens are implemented.
4. Do not store sensitive tokens in `localStorage`.
5. Do not store sensitive tokens in plaintext files.
6. `tauri-plugin-store` is a persistent key-value store, not secure credential storage.
7. If persistent secret storage is required, use an OS keychain/keyring approach or another explicitly secure storage mechanism.
8. If JWT is used, document how `JWT_SECRET` is generated, loaded, rotated, and protected.
9. Role checks use a helper such as:

```rust
guard_role(user_role, &["ADMIN"])?;
```

---

## Error Handling Convention

Use one backend error convention consistently.

Recommended pattern:

1. Define `AppError` in `src-tauri/src/error.rs`.
2. Every command returns `Result<T, AppError>` unless Tauri integration requires conversion at the boundary.
3. Error codes are stable uppercase strings:
   - `AUTH_INVALID_CREDENTIALS`
   - `AUTH_EXPIRED_TOKEN`
   - `PERMISSION_DENIED`
   - `DATABASE_ERROR`
   - `DB_CONSTRAINT`
   - `VALIDATION_ERROR`
   - `NOT_FOUND`
   - `SERIALIZATION_ERROR`
   - `IO_ERROR`
   - `INTERNAL_ERROR`
4. Do not expose raw SQLite errors, filesystem paths, secrets, or stack traces to the frontend.
5. Frontend maps error codes to i18n messages.
6. Use `?` for propagation. Do not use `unwrap()` or `expect()` inside commands.

Example command shape:

```rust
#[tauri::command]
pub fn get_customer(id: i64, state: tauri::State<AppState>) -> Result<Customer, AppError> {
    commands::customers::get_customer(id, &state)
}
```

---

## Tauri Command Rules

1. Register every command in `tauri::generate_handler![]`.
2. Keep command names stable because the frontend depends on them.
3. Use typed request/response structs for complex inputs.
4. Validate all command inputs before mutation.
5. Never trust frontend-only validation.
6. Commands that change data must:
   - validate input
   - check permissions
   - run the DB transaction
   - write audit log
   - return typed response or stable error

---

## Frontend Rules

1. Use React + TypeScript.
2. Use shadcn/ui primitives already chosen by the project.
4. Use `lucide-react` for icons.
5. Use `recharts` for charts.
6. Use `react-i18next` and `i18next` for translations.
7. Use a typed service layer for Tauri invokes.
8. Do not call `invoke()` directly from page components.
9. Do not use cookies for local desktop auth unless explicitly required.
10. Do not introduce a new UI library without approval.

Recommended invoke wrapper shape:

```ts
import { invoke } from "@tauri-apps/api/core";

export async function invokeCommand<TResponse, TArgs extends Record<string, unknown> = Record<string, never>>(
  command: string,
  args?: TArgs,
): Promise<TResponse> {
  return invoke<TResponse>(command, args);
}
```

---

## Dependency Rules

Do not install all planned dependencies upfront unless Phase 0 explicitly calls for it. Add dependencies when the feature that requires them is being implemented.

Frontend planned dependencies:

- `react-router`
- `react-i18next`
- `i18next`
- `recharts`
- `lucide-react`
- `sonner`
- `next-themes`
- shadcn/ui dependencies such as `@radix-ui/*`
- `class-variance-authority`
- `clsx`
- `tailwind-merge`
- `tailwindcss`
- `@tailwindcss/vite`
- `@tauri-apps/plugin-store`
- `@tauri-apps/plugin-dialog`
- `@tauri-apps/plugin-fs`

Rust planned dependencies:

- `rusqlite` with bundled SQLite
- `serde`
- `serde_json`
- `chrono`
- `uuid`
- `jsonwebtoken`
- `argon2`
- `sha2`
- `rand`
- `printpdf` or the final approved PDF strategy dependency
- optional: `rust_decimal` for decimal-safe calculations before persisting minor units

Before adding a dependency:

1. Check whether it already exists.
2. Confirm it matches the current phase.
3. Prefer established, maintained packages.
4. Document why it is needed if the purpose is not obvious.

---

## Security Rules

1. `tauri.conf.json` must not keep `"csp": null` for production.
2. Define a restrictive CSP before Phase 1 is considered complete.
3. Use least-privilege Tauri v2 capabilities and plugin permissions.
4. Scope filesystem access narrowly.
5. Do not store secrets in plaintext.
6. Do not log passwords, tokens, invoice-sensitive data, or filesystem secrets.
7. Validate all file paths selected through dialogs.
8. Treat imported files as untrusted input.
9. Do not enable broad shell/system access.
10. Keep auth, permissions, and audit behavior backend-enforced, not frontend-only.

---

## PDF and Arabic/RTL Rules

Invoice PDFs are a core feature and must support Arabic correctly.

Before implementing PDFs, confirm the approved strategy in `README.md`:

1. HTML/CSS rendered to PDF through a WebView/browser-style pipeline, or
2. Rust PDF generation with proper Arabic shaping and RTL layout support, or
3. Another documented strategy approved for Arabic invoices.

Rules:

1. Do not assume `printpdf` alone solves Arabic shaping.
2. Embed required fonts.
3. Verify RTL layout, Arabic shaping, numbers, currency, and mixed Arabic/English text.
4. Keep invoice PDF templates deterministic and testable.
5. Save PDFs through Tauri filesystem/dialog flows, not HTTP downloads.

---

## Audit Rules

Tracked mutations must call audit logging explicitly.

Audit entries should capture:

- actor user ID
- action code
- entity type
- entity ID
- timestamp
- before/after values when appropriate
- human-readable summary where useful

Do not audit sensitive secrets such as passwords or tokens.

---

## Price History Rules

Price changes must be atomic.

Required transaction:

1. Insert a `price_history` row.
2. Update product `current_price_minor`.
3. Write audit log if price changes are tracked.
4. Commit all changes together.

If any step fails, roll back the full transaction.

---

## Key Differences from the Original Web App

- No HTTP server.
- No CORS.
- No Swagger/OpenAPI docs unless separately generated for internal documentation.
- No web-user concurrency model.
- SQLite is embedded through `rusqlite` with bundled SQLite.
- File uploads use Tauri native dialogs, not `multer`.
- Frontend service calls use Tauri `invoke()`, not `fetch()`.
- PDFs are saved to disk and opened with the system viewer, not served by an HTTP endpoint.
- Build outputs are desktop installers/packages:
  - Windows: `.exe` / `.msi`
  - macOS: `.dmg`
  - Linux: `.deb` / `.AppImage`

---

## Do Not

Do not:

1. Use `f64`, `f32`, `REAL`, `FLOAT`, or `DOUBLE` for persisted money.
2. Use `unwrap()` or `expect()` in Tauri commands.
3. Store secrets in plaintext.
4. Store tokens in `localStorage`.
5. Treat `tauri-plugin-store` as secure secret storage.
6. Skip migrations or mutate schema ad hoc.
7. Call `invoke()` directly from React components.
8. Put business logic inside UI components.
9. Rely on frontend-only permission checks.
10. Add broad filesystem permissions.
11. Leave `"csp": null` for production.
12. Add new UI libraries without approval.
13. Implement PDF generation without confirming Arabic/RTL support.
14. Run tests unless explicitly asked.

---

## When Blocked

1. Check `README.md` for architecture decisions.
2. Check `src-tauri/migrations/001_init.sql` for the current schema.
3. Check `src-tauri/src/error.rs` for error handling patterns.
4. Check `src-tauri/src/db.rs` for database initialization patterns.
5. Check current code to see what is actually implemented.
6. If docs and code conflict, document the mismatch and make the smallest safe fix.
