# AGENTS.md — DaraERP Tauri Edition

Project: DaraERP Tauri Edition — local-first desktop ERP for steel, sheet-metal, and accessories warehouses  
Tech: React 19 + Vite + TypeScript frontend, Rust + Tauri 2 + SQLite backend  
Blueprint: Read `README.md` first for architecture, data model, migration strategy, command signatures, roadmap, and feature parity mapping.

> This file is project context for coding agents. Keep tool-specific behavior, tone preferences, and assistant style rules outside this file.

---

## Operating Rules

1. Do not run `cargo test`, `npm test`, or other test commands unless explicitly asked.
2. Before implementation, read `README.md`, especially:
   - Current Implementation State
   - Product Scope
   - Non-Negotiable Architecture Decisions
   - Data Model
   - Quick Reference
   - Feature Parity Map
   - Roadmap
3. If `README.md` and code disagree, treat the code as the current implementation and the README as the intended target. Note the mismatch before changing code.
4. Prefer small, reviewable changes over broad rewrites.
5. Do not add dependencies unless they are needed for the current task or listed in the relevant project phase.
6. Do not add inventory, purchase invoice, payment, or ledger tables before the matching specification, plan, and task list exist.

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

Use Spec Kit for new feature planning, large refactors, schema changes, and cross-document consistency checks.

Recommended next specification:

```text
Inventory and Accounts Core for Steel, Sheet Metal, and Accessories
```

---

## Feature Order

Follow the phase order defined in `README.md` Roadmap section.

1. **Phase 0: Foundation and consistency**
   - schema fixes
   - migration runner
   - scripts and checks
   - error conventions
   - security/CSP baseline
   - Arabic PDF strategy

2. **Phase 1: Master data vertical slice**
   - auth/protected shell if not complete
   - typed invoke wrapper
   - customers CRUD
   - suppliers CRUD
   - categories CRUD
   - products CRUD with steel/sheet/accessory fields
   - warehouses CRUD
   - basic audit logging

3. **Phase 2: Inventory core**
   - stock movements
   - stock balances
   - manual stock adjustments
   - product movement report
   - low-stock/reorder support
   - purchase and sale price history

4. **Phase 3: Sales and purchase invoices**
   - sales invoice lifecycle
   - purchase invoice lifecycle
   - invoice approval/cancel/return
   - automatic stock movement impact
   - tax/discount/rounding rules
   - Arabic invoice print/PDF

5. **Phase 4: Accounts, payments, and ledger**
   - customer ledger
   - supplier ledger
   - receipts
   - supplier payments
   - account statements
   - audited manual adjustments

6. **Phase 5: Reports, backup, and polish**
   - inventory report
   - sales report
   - purchase report
   - profit report
   - price changes report
   - audit viewer
   - backup/export/import
   - dashboard and charts

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
- Frontend features/pages:
  - `src/features/auth/`
  - `src/features/customers/`
  - `src/features/suppliers/`
  - `src/features/products/`
  - `src/features/inventory/`
  - `src/features/sales/`
  - `src/features/purchasing/`
  - `src/features/accounts/`
  - `src/features/reports/`
  - `src/features/settings/`
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
npm run build
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
4. SQLite schema should mirror the business model, but use desktop-appropriate storage and migration patterns.
5. All command inputs and outputs must be serializable with `serde`.
6. Commands should be thin. Put business logic in modules/services, not directly in the command function.
7. Database mutations that belong together must run in one transaction.
8. Audit logging must happen after every tracked mutation.
9. Role checks must happen before privileged mutations.
10. User-facing strings in the frontend should use i18n keys, not hard-coded English or Arabic strings.
11. Stock-changing business actions must create stock movements transactionally.
12. Account-changing business actions must create ledger transactions transactionally.

---

## Financial Precision Rules

This is a hard rule.

1. Do not persist money as `REAL`, `FLOAT`, `DOUBLE`, `f32`, or `f64`.
2. Store monetary values as integer minor units, such as Egyptian piasters.
3. Use names that make the unit obvious:
   - `current_sale_price_minor`
   - `current_purchase_price_minor`
   - `subtotal_minor`
   - `tax_total_minor`
   - `grand_total_minor`
   - `unit_price_minor`
   - `line_total_minor`
   - `discount_minor`
   - `amount_minor`
   - `debit_minor`
   - `credit_minor`
   - `price_minor`
4. Convert to display format only at the UI boundary.
5. If decimal calculations are needed, use a decimal-safe strategy such as `rust_decimal`, then persist integer minor units.
6. Invoice totals, taxes, discounts, payments, balances, ledger rows, and price history must never use floating-point storage.

Physical measurements such as weight, thickness, length, and width are not money. They still need an explicit documented precision strategy.

---

## Inventory Rules

1. Do not treat `products.stock` as the source of truth.
2. Stock must come from `stock_movements` or a projection maintained from stock movements.
3. Every stock movement must include product, warehouse, movement type, quantity/weight, source document, user, and timestamp.
4. Approving a purchase invoice must increase stock in the same transaction.
5. Approving a sales invoice must decrease stock in the same transaction.
6. Cancels, returns, and adjustments must create compensating movements instead of deleting history.
7. Manual stock adjustments require a reason and audit log.

---

## Accounts and Ledger Rules

1. Do not treat a customer or supplier balance field as the source of truth.
2. Balances must be reproducible from ledger/account transactions.
3. Sales invoices, purchase invoices, receipts, payments, returns, and manual adjustments must write ledger rows in the same transaction as the business action.
4. Manual account adjustments require a reason and audit log.
5. Account statements must be generated from ledger rows.

---

## Migration Rules

The app must use versioned migrations.

Required behavior:

1. Migrations live in `src-tauri/migrations/`.
2. Migration files use numbered names, for example:
   - `001_init.sql`
   - `002_inventory_and_accounts.sql`
   - `003_purchase_invoices.sql`
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
5. Document any command expected to perform large reads, PDF generation, imports, exports, reports, or batch stock updates.

---

## Auth and Session Rules

Planned auth stack:

- Password hashing: `argon2`
- Session/refresh strategy must be documented before protected business features.
- Admin-only commands must use role guards.
- Commands that mutate inventory, invoices, accounts, users, settings, or prices must require an authenticated user.
- Audit logs must include the acting user identity whenever available.

---

## Next Planning Step

Before implementing inventory, purchase invoices, payments, or ledger tables, create a Spec Kit specification for:

```text
Inventory and Accounts Core for Steel, Sheet Metal, and Accessories
```

The specification should define user stories, acceptance criteria, edge cases, and non-goals for:

- customers and suppliers
- products/categories/units/warehouses
- stock movement rules
- sales and purchase invoice lifecycle
- customer/supplier ledger rules
- payments and receipts
- price history
- MVP reports
- Arabic/English invoice output
