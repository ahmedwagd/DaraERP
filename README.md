# DaraERP

DaraERP is a local-first desktop ERP application built with **Tauri**, **Rust**, **SQLite**, and **React**. The project is a migration from a web-based NestJS/Prisma ERP stack into a desktop application with local persistence, typed backend commands, invoice workflows, Arabic/English UI support, and printable invoice/PDF output.

> **Current status:** foundation/scaffold stage. The architecture is defined, but most features are not implemented yet. Backend work is currently limited to the Tauri shell, database bootstrap, base error type, and empty module placeholders. Frontend work is still close to the default Tauri React template unless otherwise updated in the repo.

---

## Current Implementation State

### Implemented
- `src-tauri/src/db.rs`: database initialization, WAL mode, pragma setup.
- `src-tauri/src/error.rs`: AppError and conversion helpers.
- `src-tauri/src/lib.rs`: Tauri builder and AppState registration.
- `src-tauri/migrations/001_init.sql`: initial SQLite schema.

### Scaffolded / Empty
- `auth/`
- `commands/`
- `models/`
- `audit.rs`
- `pdf.rs`
- `seed.rs`
- `notifications_engine.rs`

### Frontend
The frontend is still the default Tauri scaffold. `App.tsx` only contains the greet demo.

### Not Installed Yet
No Tailwind, shadcn/ui, i18next, router, or frontend service layer is installed yet.

---

## Non-Negotiable Architecture Decisions

### 1. Financial precision

Do **not** persist money as floating-point values.

All persisted monetary values must use one of the following approaches:

1. **Preferred default:** `INTEGER` minor units, such as cents/piasters.
2. **Alternative:** `rust_decimal` for calculation-heavy financial logic, converted safely for storage.

Do not use SQLite `REAL`, Rust `f32`/`f64`, or JavaScript `number` as the authoritative persisted representation for invoice totals, taxes, prices, discounts, payments, balances, or ledger-like values.

Recommended naming convention:

| Meaning | Column Name | Type | Example |
|---|---|---:|---:|
| Current price | `current_price_minor` | `INTEGER NOT NULL` | `12500` = 125.00 |
| Unit price | `unit_price_minor` | `INTEGER NOT NULL` | `9999` = 99.99 |
| Subtotal | `subtotal_minor` | `INTEGER NOT NULL` | `50000` = 500.00 |
| Tax total | `tax_total_minor` | `INTEGER NOT NULL` | `7000` = 70.00 |
| Grand total | `grand_total_minor` | `INTEGER NOT NULL` | `57000` = 570.00 |
| Discount | `discount_minor` | `INTEGER NOT NULL DEFAULT 0` | `1000` = 10.00 |
| Price history value | `price_minor` | `INTEGER NOT NULL` | `12500` = 125.00 |

Application code may format these values for display, but database writes must store minor units.

### 2. Migration strategy

The app must use a versioned migration runner before Phase 1 feature work begins.

Do not execute a single `include_str!` SQL file on every startup as the long-term strategy. Instead:

1. Create a `schema_migrations` table.
2. Store every migration as a versioned file.
3. Run only unapplied migrations.
4. Execute each migration inside a transaction.
5. Record the applied version and timestamp.
6. Fail startup safely if a migration fails.

Suggested table:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Suggested file naming:

```text
src-tauri/migrations/
  001_init.sql
  002_add_invoice_status_index.sql
  003_add_company_logo.sql
```

### 3. Arabic invoice/PDF strategy

Arabic invoice output must be designed before invoice generation is implemented.

Arabic PDFs require correct RTL layout, shaping, font embedding, line breaking, and printable output. Do not assume a basic PDF crate alone will produce production-grade Arabic invoices.

Acceptable strategies:

| Strategy | Use When | Notes |
|---|---|---|
| HTML/CSS invoice rendered through a browser/WebView-compatible pipeline | Fastest reliable path | Usually best for Arabic layout fidelity |
| Rust PDF pipeline with explicit shaping/layout support | Maximum backend control | Requires careful font and shaping implementation |
| Frontend-generated PDF | Useful for quick prototype | Must be validated for print consistency and Arabic text |

A final choice must be documented before invoice PDF work starts.

### 4. Tauri security baseline

Production builds must define a security baseline:

- `csp` must not remain `null` for production.
- Tauri v2 plugin permissions/capabilities must be least-privilege.
- Filesystem access must be scoped.
- Sensitive tokens/secrets must not be stored in plaintext app config or a generic key-value store.
- Backend commands must validate authorization and input.
- Frontend code must not call privileged commands directly from arbitrary components.

---

## Project Structure

Recommended target structure:

```text
DaraERP/
  README.md
  AGENTS.md
  package.json
  src/
    App.tsx
    main.tsx
    index.css
    app/
      router.tsx
      providers.tsx
    components/
      ui/
      layout/
    features/
      auth/
      customers/
      products/
      invoices/
      settings/
      audit/
      notifications/
    lib/
      tauri.ts
      money.ts
      errors.ts
      i18n.ts
    shared/
      types/
      utils/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    migrations/
      001_init.sql
    src/
      lib.rs
      db.rs
      error.rs
      auth/
      commands/
      models/
      seed.rs
      audit.rs
      pdf.rs
      notifications_engine.rs
```

---

## Setup

### Prerequisites

Install:

- Node.js LTS
- npm or pnpm, depending on the repo standard
- Rust stable toolchain
- Tauri system dependencies for your operating system
- SQLite tooling, optional but recommended

### Install dependencies

From the project root:

```bash
npm install
```

### Run the app in development

From the project root:

```bash
npm run tauri dev
```

### Build the frontend

```bash
npm run build
```

### Run frontend linting

```bash
npm run lint
```

`package.json` must define this script if docs or agents reference it.

### Check Rust backend

From the project root:

```bash
cd src-tauri && cargo check
```

### Check Rust formatting

From the project root:

```bash
cd src-tauri && cargo fmt -- --check
```

---

## Required Scripts

`package.json` should include at least:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint .",
    "tauri": "tauri"
  }
}
```

Adjust the lint command if the repo uses a different ESLint or TypeScript configuration.

---

## Backend Architecture

### Command pattern

Tauri commands should be thin IPC entry points. They should:

1. Validate input.
2. Authorize the current user/session.
3. Call domain/database logic.
4. Return typed data or `AppError`.
5. Avoid business logic directly in the command body when it can live in a service/domain function.

Recommended command shape:

```rust
#[tauri::command]
pub async fn create_customer(
    state: tauri::State<'_, AppState>,
    input: CreateCustomerInput,
) -> Result<CustomerDto, AppError> {
    // validate, authorize, execute, return
}
```

### Error convention

Use a single backend error type, such as `AppError`, across all commands.

Recommended rules:

- Commands return `Result<T, AppError>`.
- Error codes are stable uppercase strings, e.g. `VALIDATION_ERROR`, `UNAUTHORIZED`, `NOT_FOUND`.
- Frontend maps error codes to i18n keys.
- Do not expose raw SQL errors directly to users.
- Log internal details where appropriate, but return safe messages to the UI.

Example categories:

| Code | Meaning |
|---|---|
| `VALIDATION_ERROR` | Invalid user input |
| `UNAUTHORIZED` | User is not authenticated |
| `FORBIDDEN` | User lacks permission |
| `NOT_FOUND` | Requested record does not exist |
| `CONFLICT` | Duplicate or conflicting state |
| `DATABASE_ERROR` | Safe wrapper for database failure |
| `INTERNAL_ERROR` | Unexpected failure |

### Database access

A single `Mutex<Connection>` is acceptable as an early scaffold for a single-user desktop app, but it has tradeoffs:

- It serializes all database access.
- It can block async command execution.
- Long-running operations can make the UI feel slow.

Before the app grows, choose one of these strategies:

1. Move blocking DB work into `spawn_blocking`.
2. Use a connection pool such as `deadpool`/SQLite-compatible pooling.
3. Keep the mutex explicitly and document that the app is single-user/local-only with low concurrency.

The chosen approach must be documented before heavy CRUD work begins.

---

## Data Model

This section describes the intended schema. Keep it synchronized with `src-tauri/migrations/`.

> **Financial precision warning:**
> The current initial schema uses `REAL` for monetary values. This is acceptable only as a temporary scaffold state. Before Phase 1 is considered complete, all monetary values must be migrated to INTEGER minor units, using Egyptian piasters.

### Users

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `username` | `TEXT` | Unique login identifier |
| `password_hash` | `TEXT` | Argon2 or approved password hash |
| `role` | `TEXT` | Role key, e.g. `admin`, `manager`, `staff` |
| `is_active` | `INTEGER` | Boolean as 0/1 |
| `created_at` | `TEXT` | ISO timestamp |
| `updated_at` | `TEXT` | ISO timestamp |

### Customers

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `name` | `TEXT` | Required |
| `phone` | `TEXT` | Optional |
| `email` | `TEXT` | Optional |
| `tax_number` | `TEXT` | Optional |
| `address` | `TEXT` | Optional |
| `notes` | `TEXT` | Optional |
| `created_at` | `TEXT` | ISO timestamp |
| `updated_at` | `TEXT` | ISO timestamp |

### Categories

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `name` | `TEXT` | Unique or scoped unique |
| `description` | `TEXT` | Optional |
| `created_at` | `TEXT` | ISO timestamp |
| `updated_at` | `TEXT` | ISO timestamp |

### Products

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `category_id` | `TEXT` | FK to categories |
| `name` | `TEXT` | Required |
| `sku` | `TEXT` | Optional unique code |
| `current_price_minor` | `INTEGER` | Required; never `REAL` |
| `is_active` | `INTEGER` | Boolean as 0/1 |
| `created_at` | `TEXT` | ISO timestamp |
| `updated_at` | `TEXT` | ISO timestamp |

### Price History

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `product_id` | `TEXT` | FK to products |
| `price_minor` | `INTEGER` | Required; never `REAL` |
| `effective_from` | `TEXT` | ISO timestamp/date |
| `changed_by_user_id` | `TEXT` | FK to users |
| `created_at` | `TEXT` | ISO timestamp |

### Invoices

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `invoice_number` | `TEXT` | Unique business number |
| `customer_id` | `TEXT` | FK to customers |
| `status` | `TEXT` | `draft`, `issued`, `paid`, `cancelled`, etc. |
| `subtotal_minor` | `INTEGER` | Required; never `REAL` |
| `discount_minor` | `INTEGER` | Required; default 0 |
| `tax_total_minor` | `INTEGER` | Required; never `REAL` |
| `grand_total_minor` | `INTEGER` | Required; never `REAL` |
| `issued_at` | `TEXT` | Optional until issued |
| `due_at` | `TEXT` | Optional |
| `created_by_user_id` | `TEXT` | FK to users |
| `created_at` | `TEXT` | ISO timestamp |
| `updated_at` | `TEXT` | ISO timestamp |

### Invoice Lines

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `invoice_id` | `TEXT` | FK to invoices |
| `product_id` | `TEXT` | FK to products; nullable only if custom line items are supported |
| `description` | `TEXT` | Display label |
| `quantity` | `INTEGER` or decimal strategy | Decide based on product units |
| `unit_price_minor` | `INTEGER` | Required; never `REAL` |
| `discount_minor` | `INTEGER` | Required; default 0 |
| `tax_rate_bps` | `INTEGER` | Basis points, e.g. 1400 = 14% |
| `line_total_minor` | `INTEGER` | Required; never `REAL` |
| `created_at` | `TEXT` | ISO timestamp |

### Audit Log

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | UUID |
| `actor_user_id` | `TEXT` | FK to users |
| `entity_type` | `TEXT` | e.g. `invoice`, `customer` |
| `entity_id` | `TEXT` | Record ID |
| `action` | `TEXT` | e.g. `create`, `update`, `delete`, `issue` |
| `before_json` | `TEXT` | Optional JSON snapshot |
| `after_json` | `TEXT` | Optional JSON snapshot |
| `created_at` | `TEXT` | ISO timestamp |

### Company Settings

| Field | Type | Notes |
|---|---|---|
| `id` | `TEXT` | Usually single row |
| `company_name` | `TEXT` | Required |
| `tax_number` | `TEXT` | Optional |
| `address` | `TEXT` | Optional |
| `phone` | `TEXT` | Optional |
| `email` | `TEXT` | Optional |
| `logo_path` | `TEXT` | Must be scoped through Tauri FS rules |
| `currency_code` | `TEXT` | e.g. `EGP` |
| `updated_at` | `TEXT` | ISO timestamp |

---

## Quick Reference: Backend Invoke Signatures

Use a single frontend invoke wrapper instead of calling `invoke` directly from components.

Example wrapper location:

```text
src/lib/tauri.ts
```

### Auth

| Command | Input | Output | Notes |
|---|---|---|---|
| `auth_login` | `{ username, password }` | `SessionDto` | Creates local session/token |
| `auth_logout` | `{}` | `void` | Clears session |
| `auth_me` | `{}` | `UserDto` | Returns current user |
| `auth_refresh` | `{}` | `SessionDto` | Only if refresh flow is used |

### Customers

| Command | Input | Output |
|---|---|---|
| `customers_list` | `{ query?, page?, page_size? }` | `Paginated<CustomerDto>` |
| `customers_get` | `{ id }` | `CustomerDto` |
| `customers_create` | `CreateCustomerInput` | `CustomerDto` |
| `customers_update` | `{ id, patch }` | `CustomerDto` |
| `customers_delete` | `{ id }` | `void` |

### Products and Categories

| Command | Input | Output |
|---|---|---|
| `categories_list` | `{}` | `CategoryDto[]` |
| `categories_create` | `CreateCategoryInput` | `CategoryDto` |
| `products_list` | `{ query?, category_id?, page?, page_size? }` | `Paginated<ProductDto>` |
| `products_get` | `{ id }` | `ProductDto` |
| `products_create` | `CreateProductInput` | `ProductDto` |
| `products_update` | `{ id, patch }` | `ProductDto` |
| `products_set_price` | `{ id, price_minor }` | `ProductDto` |
| `products_price_history` | `{ id }` | `PriceHistoryDto[]` |

### Invoices

| Command | Input | Output |
|---|---|---|
| `invoices_list` | `{ status?, query?, page?, page_size? }` | `Paginated<InvoiceSummaryDto>` |
| `invoices_get` | `{ id }` | `InvoiceDto` |
| `invoices_create_draft` | `CreateInvoiceDraftInput` | `InvoiceDto` |
| `invoices_update_draft` | `{ id, patch }` | `InvoiceDto` |
| `invoices_issue` | `{ id }` | `InvoiceDto` |
| `invoices_cancel` | `{ id, reason? }` | `InvoiceDto` |
| `invoices_generate_pdf` | `{ id }` | `PdfResultDto` |

### Settings, Audit, Notifications

| Command | Input | Output |
|---|---|---|
| `settings_get_company` | `{}` | `CompanySettingsDto` |
| `settings_update_company` | `UpdateCompanySettingsInput` | `CompanySettingsDto` |
| `audit_list` | `{ entity_type?, entity_id?, page?, page_size? }` | `Paginated<AuditLogDto>` |
| `notifications_list` | `{ unread_only? }` | `NotificationDto[]` |
| `notifications_mark_read` | `{ id }` | `void` |

---

## Frontend Architecture

### Rules

- Components should not call Tauri `invoke` directly.
- Use `src/lib/tauri.ts` or feature-specific service files.
- All backend errors should be normalized through `src/lib/errors.ts`.
- Money values should be represented as minor units in application state and formatted only for display.
- Feature pages should live under `src/features/<feature>/`.
- Shared UI primitives should live under `src/components/ui/`.

### Planned dependencies

These dependencies should be added when their phase starts, unless the team chooses to install them in Phase 0.

| Dependency | Phase | Purpose |
|---|---|---|
| `react-router` | Phase 1 | Routing |
| `react-i18next`, `i18next` | Phase 1 | Arabic/English localization |
| `lucide-react` | Phase 1 | Icons |
| `sonner` | Phase 1 | Toasts |
| `tailwindcss`, `@tailwindcss/vite` | Phase 1 | Styling |
| `@radix-ui/*` | Phase 1/2 | UI primitives |
| `class-variance-authority`, `clsx`, `tailwind-merge` | Phase 1 | shadcn-style utilities |
| `recharts` | Phase 3 | Dashboard/report charts |
| `@tauri-apps/plugin-store` | As needed | Non-sensitive local app settings |
| `@tauri-apps/plugin-dialog` | As needed | File dialogs |
| `@tauri-apps/plugin-fs` | As needed | Scoped filesystem access |

---

## Feature Parity Map

This project is a Tauri desktop migration of a previous NestJS/Prisma system. The original source code is not present in this repository, so this table is reference-only and should not be treated as an executable migration checklist.

| Legacy NestJS/Web Area | New Tauri/Rust Area | Frontend Area | Status |
|---|---|---|---|
| Auth module | `src-tauri/src/auth/`, `src-tauri/src/commands/auth.rs` | `src/features/auth/` | Planned |
| Users/roles | `models/user.rs`, `commands/users.rs` | `src/features/settings/users/` | Planned |
| Customers | `commands/customers.rs` | `src/features/customers/` | Planned |
| Categories | `commands/categories.rs` | `src/features/products/categories/` | Planned |
| Products | `commands/products.rs` | `src/features/products/` | Planned |
| Price history | Product service transaction logic | Product detail/history UI | Planned |
| Invoices | `commands/invoices.rs`, invoice domain service | `src/features/invoices/` | Planned |
| PDF generation | `pdf.rs` or selected PDF strategy | Invoice print/preview UI | Architecture decision required |
| Notifications | `notifications_engine.rs` | Bell/dropdown UI | Planned |
| Audit logging | `audit.rs` | `src/features/audit/` | Planned |
| Company settings | `commands/settings.rs` | `src/features/settings/company/` | Planned |
| Dashboard/reports | Query/report services | `src/features/dashboard/` | Planned |

When the legacy project is available, replace generic entries with exact paths, such as:

```text
../DaraERP-legacy/api/src/<module>
../DaraERP-legacy/client/src/<feature>
```

---

## Roadmap

### Phase 0: Foundation and consistency

Goal: remove contradictions and make the repo safe for feature work.

Tasks:

1. Resolve all broken documentation references.
2. Replace default inner README content.
3. Fix the schema to use integer minor units for money.
4. Add a versioned migration runner.
5. Add required build/lint/check scripts.
6. Define backend error convention in code and docs.
7. Define auth/session strategy.
8. Define Arabic invoice/PDF strategy.
9. Define Tauri CSP and plugin capability baseline.
10. Document DB concurrency strategy.

Definition of Done:

- `npm run build` works.
- `npm run lint` exists and runs.
- `cd src-tauri && cargo check` works.
- `cd src-tauri && cargo fmt -- --check` works.
- No docs reference missing sections or missing paths.
- No money fields in migrations use SQLite `REAL`.
- Migration runner applies only unapplied migrations.
- CSP and capability strategy are documented.
- Phase 1 dependencies are clearly marked or installed.

### Phase 1: Minimal working vertical slice

Goal: prove the full app pattern from UI to DB with one real feature.

Tasks:

1. Implement migration runner.
2. Implement seed data.
3. Implement login/session flow.
4. Implement protected app shell.
5. Implement typed invoke wrapper.
6. Implement customers CRUD end to end.
7. Add basic audit logging for customer create/update/delete.
8. Add basic frontend error handling and i18n mapping.

Definition of Done:

- User can log in.
- Protected routes block unauthenticated access.
- User can create, list, edit, and delete customers.
- Database changes persist across app restarts.
- Errors are returned as `AppError` and displayed safely in the UI.
- Audit records are captured for the vertical slice.

### Phase 2: Invoicing core

Goal: implement business-critical ERP workflows.

Tasks:

1. Products and categories CRUD.
2. Price history with transactional updates.
3. Invoice draft/create/update workflows.
4. Invoice issue/cancel workflow.
5. Tax, discount, and rounding rules.
6. PDF/print generation according to selected Arabic strategy.
7. Company settings and logo handling.

Definition of Done:

- User can manage customers, products, and categories.
- User can create and issue invoices.
- Invoice totals are calculated using approved money precision rules.
- Invoice PDF/print output handles Arabic and English correctly.
- Price changes are recorded in history.
- Company settings appear on invoices.

### Phase 3: Supporting ERP features

Goal: add operational visibility and polish.

Tasks:

1. Notifications engine and bell UI.
2. Audit log viewer.
3. Permission matrix UI.
4. Dashboard and reports.
5. Backup/export/import strategy.
6. Additional tests and quality gates.

Definition of Done:

- Users can review important activity and notifications.
- Admins can inspect audit history.
- Permissions are visible and enforceable.
- Reports show accurate data.
- Backup/export/import story is documented and tested.

---

## Security Notes

### CSP

Do not leave production `tauri.conf.json` with:

```json
"csp": null
```

Define a restrictive CSP before Phase 1 is completed. During development, document any temporary relaxations.

### Secrets and sessions

- Do not hard-code JWT secrets, encryption keys, or admin credentials.
- Do not store persistent sensitive tokens in plaintext.
- Use environment variables, OS keychain integration, or a documented local-session model.
- If JWTs are retained from the web architecture, document why they are needed in a local desktop app.

### Tauri capabilities

When using plugins such as `fs`, `dialog`, or `store`:

- Add only the permissions required by the feature.
- Scope filesystem access to specific base directories.
- Do not grant broad filesystem permissions without explicit justification.
- Review capabilities before release builds.

---

## Implementation Guidelines

### Do

- Keep migrations small and versioned.
- Use transactions for multi-step writes.
- Store money in minor units.
- Use typed DTOs for Tauri command inputs/outputs.
- Keep commands thin and domain logic testable.
- Map backend error codes to frontend translations.
- Validate and authorize every privileged command.
- Keep docs synchronized with actual scripts and paths.

### Do Not

- Do not use `REAL`, `f64`, or JavaScript floating-point values as persisted money.
- Do not call `unwrap()` or `expect()` inside Tauri commands for recoverable errors.
- Do not expose raw SQL errors to users.
- Do not run a full init SQL file on every startup as a migration strategy.
- Do not store secrets in plaintext files or generic app settings.
- Do not leave production CSP as `null`.
- Do not call `invoke` directly from React components.
- Do not add broad Tauri filesystem permissions without a scoped need.
- Do not reference legacy directories unless the exact path exists.

---

## Recommended Immediate Fixes

1. Update `001_init.sql` money fields from `REAL` to `INTEGER` minor-unit columns.
2. Implement a real migration runner with `schema_migrations`.
3. Add this README's `Data Model`, `Quick Reference`, and `Feature Parity Map` sections if missing.
4. Update `AGENTS.md` so all section references exist.
5. Add `npm run lint` or remove references to it.
6. Remove tool-specific behavior directives from project docs.
7. Define CSP and Tauri capability baseline.

