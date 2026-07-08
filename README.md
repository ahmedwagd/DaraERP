# DaraERP

DaraERP is a local-first desktop ERP application for **steel, sheet-metal, and accessories warehouses**. It is built with **Tauri**, **Rust**, **SQLite**, and **React**.

The product scope is focused on small and medium steel trading operations:

- customer and supplier records
- customer and supplier account statements
- sales invoices
- purchase invoices
- product catalog for steel, sheet metal, and accessories
- warehouse stock tracking
- stock movements and adjustments
- price changes and price history
- Arabic/English UI support
- printable invoices and operational reports

> **Current status:** foundation/scaffold stage. The architecture is defined, but business features are not fully implemented yet. Backend work currently includes the Tauri shell, database bootstrap, versioned migration runner, base error type, authentication/session groundwork, seed support, and module placeholders. Frontend work is still early and should be treated as incomplete unless the current code proves otherwise.

---

## Current Implementation State

### Implemented / Started

- `src-tauri/src/db.rs`: database initialization, WAL mode, pragma setup, and versioned migration execution.
- `src-tauri/src/error.rs`: AppError and conversion helpers.
- `src-tauri/src/lib.rs`: Tauri builder and AppState registration.
- `src-tauri/migrations/001_init.sql`: initial SQLite schema with integer minor units for money and `schema_migrations`.
- `package.json`: basic Vite/Tauri scripts, including `lint`.
- `src-tauri/tauri.conf.json`: CSP is defined and must not be relaxed without justification.

### Scaffolded / Incomplete

- `auth/`
- `commands/`
- `models/`
- `audit.rs`
- `pdf.rs`
- `seed.rs`
- `notifications_engine.rs`
- Frontend feature pages and service wrappers

### Important Gap

The current schema still represents a generic invoicing foundation. The target business domain requires additional versioned migrations for suppliers, warehouses, inventory movements, purchase invoices, payments, and ledger/account transactions. Do not add these tables casually; add them through Spec Kit planning and versioned migrations.

---

## Product Scope

DaraERP should solve the daily workflow for a steel and sheet-metal warehouse:

1. Register customers and suppliers.
2. Define categories, products, units, and warehouses.
3. Track product measurements such as thickness, dimensions, weight, unit type, and notes.
4. Record purchase invoices from suppliers.
5. Record sales invoices to customers.
6. Automatically create stock movements when approved invoices affect stock.
7. Track customer receivables and supplier payables through ledger transactions.
8. Record payments and receipts.
9. Track purchase and sale price changes.
10. Print invoices and show operational reports.

The system is local-first and desktop-oriented. Networked multi-branch operation is not part of the first implementation unless a future specification explicitly adds it.

---

## Non-Negotiable Architecture Decisions

### 1. Financial precision

Do **not** persist money as floating-point values.

All persisted monetary values must use one of the following approaches:

1. **Preferred default:** `INTEGER` minor units, such as Egyptian piasters.
2. **Alternative:** `rust_decimal` for calculation-heavy financial logic, converted safely for storage.

Do not use SQLite `REAL`, Rust `f32`/`f64`, or JavaScript floating-point values as the authoritative persisted representation for invoice totals, taxes, prices, discounts, payments, balances, or ledger values.

Recommended naming convention:

| Meaning | Column Name | Type | Example |
|---|---|---:|---:|
| Current sale price | `current_sale_price_minor` | `INTEGER` | `12500` = 125.00 |
| Current purchase price | `current_purchase_price_minor` | `INTEGER` | `9999` = 99.99 |
| Unit price | `unit_price_minor` | `INTEGER` | `9999` = 99.99 |
| Subtotal | `subtotal_minor` | `INTEGER` | `50000` = 500.00 |
| Tax total | `tax_total_minor` | `INTEGER` | `7000` = 70.00 |
| Grand total | `grand_total_minor` | `INTEGER` | `57000` = 570.00 |
| Discount | `discount_minor` | `INTEGER DEFAULT 0` | `1000` = 10.00 |
| Payment amount | `amount_minor` | `INTEGER` | `250000` = 2,500.00 |
| Ledger debit | `debit_minor` | `INTEGER DEFAULT 0` | `10000` = 100.00 |
| Ledger credit | `credit_minor` | `INTEGER DEFAULT 0` | `10000` = 100.00 |
| Price history value | `price_minor` | `INTEGER` | `12500` = 125.00 |

Physical measurements such as thickness, length, width, and weight are not money. They may use a documented decimal strategy. If exact calculations are required, prefer scaled integers or decimal-safe handling rather than vague floating-point logic.

### 2. Inventory truth comes from movements

Do not store stock as a manually edited number only on `products`.

Stock balance must be derived from `stock_movements` or maintained as a transactional projection updated from movements.

Every stock change must have:

- product
- warehouse
- movement type
- quantity or weight
- source document type and ID
- timestamp
- actor/user when available

Invoice approval, purchase approval, returns, transfers, and adjustments must create stock movements in the same transaction as the business action that caused them.

### 3. Accounts truth comes from ledger transactions

Do not store customer or supplier balances as isolated editable fields.

Balances must come from account/ledger transactions or from a transactional projection derived from the ledger.

Examples:

- sales invoice: debit customer
- customer receipt: credit customer
- sales return: credit customer
- purchase invoice: credit supplier or debit payable strategy depending on ledger convention
- supplier payment: debit supplier
- manual settlement: explicit adjustment with reason and audit log

A customer or supplier account statement must be reproducible from ledger rows.

### 4. Versioned migrations

The app must use a versioned migration runner.

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
7. Do not run a full init SQL file on every startup as a long-term strategy.
8. Never edit an already-applied migration unless the database is disposable scaffold data.

### 5. Arabic invoice/PDF strategy

Arabic invoice output must be designed before invoice generation is implemented.

Arabic PDFs require correct RTL layout, shaping, font embedding, line breaking, and printable output. Do not assume a basic PDF crate alone will produce production-grade Arabic invoices.

Acceptable strategies:

| Strategy | Use When | Notes |
|---|---|---|
| HTML/CSS invoice rendered through a browser/WebView-compatible pipeline | Fastest reliable path | Usually best for Arabic layout fidelity |
| Rust PDF pipeline with explicit shaping/layout support | Maximum backend control | Requires careful font and shaping implementation |
| Frontend-generated PDF | Useful for quick prototype | Must be validated for print consistency and Arabic text |

A final choice must be documented before invoice PDF work starts.

### 6. Tauri security baseline

Production builds must define a security baseline:

- `csp` must not remain `null`.
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
      suppliers/
      products/
      inventory/
      sales/
      purchasing/
      accounts/
      reports/
      settings/
      audit/
      notifications/
    lib/
      tauri.ts
      money.ts
      measurements.ts
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
      002_inventory_and_accounts.sql
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

```bash
npm install
```

### Run the app in development

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

### Check Rust backend

```bash
cd src-tauri && cargo check
```

### Check Rust formatting

```bash
cd src-tauri && cargo fmt -- --check
```

### Check Rust lints

```bash
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
```

Do not run test commands unless explicitly requested:

```bash
npm test
npm run test
cargo test
```

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

The chosen approach must be documented before heavy CRUD, reports, imports, PDF generation, or bulk stock updates.

---

## Data Model

This section describes the intended target schema. Keep it synchronized with `src-tauri/migrations/` as implementation progresses.

> **Implementation note:** `001_init.sql` is the foundation schema. It already uses integer minor units for persisted money. Domain-specific tables for inventory, purchases, payments, and ledger/account statements should be added through later versioned migrations after a specification is approved.

### Existing Foundation Tables

- `users`
- `refresh_tokens`
- `customers`
- `categories`
- `products`
- `price_history`
- generic `invoices` / `invoice_items`
- `notifications`
- `audit_logs`
- `company_settings`

The generic invoice tables may later be split or superseded by `sales_invoices` and `purchase_invoices` through versioned migrations.

### Target Domain Tables

The steel inventory ERP target should eventually include:

- `customers`
- `suppliers`
- `categories`
- `products`
- `warehouses`
- `stock_movements`
- `price_history`
- `sales_invoices`
- `sales_invoice_lines`
- `purchase_invoices`
- `purchase_invoice_lines`
- `payments`
- `account_transactions`
- `audit_logs`
- `company_settings`

### Customers

Customers should support contact details, active/inactive status, and account statement generation through ledger rows. Avoid storing a manually edited balance as the source of truth.

### Suppliers

Suppliers should mirror customer contact fields and support supplier account statements. Purchase invoices and supplier payments should affect supplier ledger rows.

### Products

Products should support steel and sheet-metal attributes without forcing all products into the same physical shape:

- `product_type`: `steel`, `sheet`, `accessory`, `service`, etc.
- `unit`: `kg`, `ton`, `sheet`, `piece`, `meter`, etc.
- optional thickness, width, length, and weight fields
- purchase and sale price tracking
- stock-tracked flag, because services may not affect inventory

### Warehouses

Warehouses represent physical storage locations. Start with one default warehouse, but design the schema so multiple warehouses can be added without refactoring.

### Stock Movements

`stock_movements` is the inventory source of truth. It should record every purchase, sale, return, adjustment, and transfer.

### Price History

Price history should track purchase and sale price changes separately. Each change should record old/new price, reason, effective date, and actor.

### Sales Invoices

Sales invoices should support draft, approval, cancellation, and return flows. Approval should create stock movements and customer ledger entries transactionally.

### Purchase Invoices

Purchase invoices should support draft, approval, cancellation, and return flows. Approval should create stock movements and supplier ledger entries transactionally.

### Payments and Receipts

Payments and receipts should create ledger entries. They may optionally be linked to invoices, but the account statement must still be reproducible from ledger rows.

### Account Transactions / Ledger

Ledger rows must include party type, party ID, debit, credit, source document, description, actor, and timestamp.

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
| `auth_login` | `{ email, password }` | `SessionDto` | Creates local session/token |
| `auth_logout` | `{}` | `void` | Clears session |
| `auth_me` | `{}` | `UserDto` | Returns current user |
| `auth_refresh` | `{}` | `SessionDto` | Only if refresh flow is used |

### Master Data

| Command | Input | Output |
|---|---|---|
| `customers_list` | `{ query?, is_active?, page?, page_size? }` | `Paginated<CustomerDto>` |
| `customers_get` | `{ id }` | `CustomerDto` |
| `customers_create` | `CreateCustomerInput` | `CustomerDto` |
| `customers_update` | `{ id, patch }` | `CustomerDto` |
| `suppliers_list` | `{ query?, is_active?, page?, page_size? }` | `Paginated<SupplierDto>` |
| `suppliers_create` | `CreateSupplierInput` | `SupplierDto` |
| `categories_list` | `{}` | `CategoryDto[]` |
| `categories_create` | `CreateCategoryInput` | `CategoryDto` |
| `products_list` | `{ query?, category_id?, page?, page_size? }` | `Paginated<ProductDto>` |
| `products_get` | `{ id }` | `ProductDto` |
| `products_create` | `CreateProductInput` | `ProductDto` |
| `products_update` | `{ id, patch }` | `ProductDto` |
| `warehouses_list` | `{}` | `WarehouseDto[]` |
| `warehouses_create` | `CreateWarehouseInput` | `WarehouseDto` |

### Inventory and Price History

| Command | Input | Output |
|---|---|---|
| `inventory_balance` | `{ warehouse_id?, product_id?, page?, page_size? }` | `Paginated<StockBalanceDto>` |
| `inventory_movements` | `{ product_id?, warehouse_id?, from?, to? }` | `StockMovementDto[]` |
| `inventory_adjust` | `CreateStockAdjustmentInput` | `StockMovementDto` |
| `products_set_price` | `{ id, price_type, price_minor, reason? }` | `ProductDto` |
| `products_price_history` | `{ id, price_type? }` | `PriceHistoryDto[]` |

### Sales

| Command | Input | Output |
|---|---|---|
| `sales_invoices_list` | `{ status?, customer_id?, query?, page?, page_size? }` | `Paginated<SalesInvoiceSummaryDto>` |
| `sales_invoices_get` | `{ id }` | `SalesInvoiceDto` |
| `sales_invoices_create_draft` | `CreateSalesInvoiceDraftInput` | `SalesInvoiceDto` |
| `sales_invoices_update_draft` | `{ id, patch }` | `SalesInvoiceDto` |
| `sales_invoices_approve` | `{ id, payment? }` | `SalesInvoiceDto` |
| `sales_invoices_cancel` | `{ id, reason }` | `SalesInvoiceDto` |
| `sales_invoices_generate_pdf` | `{ id }` | `PdfResultDto` |

### Purchasing

| Command | Input | Output |
|---|---|---|
| `purchase_invoices_list` | `{ status?, supplier_id?, query?, page?, page_size? }` | `Paginated<PurchaseInvoiceSummaryDto>` |
| `purchase_invoices_get` | `{ id }` | `PurchaseInvoiceDto` |
| `purchase_invoices_create_draft` | `CreatePurchaseInvoiceDraftInput` | `PurchaseInvoiceDto` |
| `purchase_invoices_update_draft` | `{ id, patch }` | `PurchaseInvoiceDto` |
| `purchase_invoices_approve` | `{ id, payment? }` | `PurchaseInvoiceDto` |
| `purchase_invoices_cancel` | `{ id, reason }` | `PurchaseInvoiceDto` |

### Accounts

| Command | Input | Output |
|---|---|---|
| `accounts_customer_statement` | `{ customer_id, from?, to? }` | `AccountStatementDto` |
| `accounts_supplier_statement` | `{ supplier_id, from?, to? }` | `AccountStatementDto` |
| `payments_create_receipt` | `CreateReceiptInput` | `PaymentDto` |
| `payments_create_supplier_payment` | `CreateSupplierPaymentInput` | `PaymentDto` |
| `accounts_adjust` | `CreateAccountAdjustmentInput` | `AccountTransactionDto` |

### Reports, Settings, Audit, Notifications

| Command | Input | Output |
|---|---|---|
| `reports_inventory` | `{ warehouse_id?, category_id? }` | `InventoryReportDto` |
| `reports_sales` | `{ from, to, customer_id? }` | `SalesReportDto` |
| `reports_purchases` | `{ from, to, supplier_id? }` | `PurchaseReportDto` |
| `reports_profit` | `{ from, to }` | `ProfitReportDto` |
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
- Measurements should have explicit units and formatting helpers.
- Feature pages should live under `src/features/<feature>/`.
- Shared UI primitives should live under `src/components/ui/`.
- User-facing strings should use i18n keys, not hard-coded English or Arabic.

---

## Feature Parity Map

This project is a Tauri desktop migration of a previous NestJS/Prisma system, but the current target is now a focused steel inventory ERP. Treat this table as target orientation, not as an executable migration checklist.

| Business Area | New Tauri/Rust Area | Frontend Area | Status |
|---|---|---|---|
| Auth/session | `src-tauri/src/auth/`, `src-tauri/src/commands/auth.rs` | `src/features/auth/` | Started/Planned |
| Users/roles | `models/user.rs`, `commands/users.rs` | `src/features/settings/users/` | Planned |
| Customers | `commands/customers.rs` | `src/features/customers/` | Planned |
| Suppliers | `commands/suppliers.rs` | `src/features/suppliers/` | Planned |
| Categories/products/units | `commands/products.rs`, `commands/categories.rs` | `src/features/products/` | Planned |
| Warehouses | `commands/warehouses.rs` | `src/features/inventory/warehouses/` | Planned |
| Stock movements/balances | Inventory domain service | `src/features/inventory/` | Planned |
| Price history | Product service transaction logic | Product detail/history UI | Planned |
| Sales invoices | `commands/sales_invoices.rs` | `src/features/sales/` | Planned |
| Purchase invoices | `commands/purchase_invoices.rs` | `src/features/purchasing/` | Planned |
| Payments and receipts | `commands/payments.rs` | `src/features/accounts/payments/` | Planned |
| Account statements | Ledger/report services | `src/features/accounts/` | Planned |
| PDF generation | `pdf.rs` or selected PDF strategy | Invoice print/preview UI | Architecture decision required |
| Reports | Query/report services | `src/features/reports/` | Planned |
| Notifications | `notifications_engine.rs` | Bell/dropdown UI | Planned |
| Audit logging | `audit.rs` | `src/features/audit/` | Planned |
| Company settings | `commands/settings.rs` | `src/features/settings/company/` | Planned |

---

## Roadmap

### Phase 0: Foundation and consistency

Goal: remove contradictions and make the repo safe for feature work.

Tasks:

1. Resolve all broken documentation references.
2. Fix the schema to use integer minor units for money.
3. Add a versioned migration runner.
4. Add required build/lint/check scripts.
5. Define backend error convention in code and docs.
6. Define auth/session strategy.
7. Define Arabic invoice/PDF strategy.
8. Define Tauri CSP and plugin capability baseline.
9. Document DB concurrency strategy.

Definition of Done:

- `npm run build` works.
- `npm run lint` exists and runs.
- `cd src-tauri && cargo check` works.
- `cd src-tauri && cargo fmt -- --check` works.
- `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings` works.
- No docs reference missing sections or missing paths.
- No money fields in migrations use SQLite `REAL`.
- Migration runner applies only unapplied migrations.
- CSP and capability strategy are documented.

### Phase 1: Master data vertical slice

Goal: prove the full app pattern from UI to DB using the master data needed before inventory and invoices.

Tasks:

1. Implement protected app shell and typed invoke wrapper.
2. Implement login/session flow if not complete.
3. Implement customers CRUD.
4. Implement suppliers CRUD.
5. Implement categories CRUD.
6. Implement products CRUD with steel/sheet/accessory fields.
7. Implement warehouses CRUD.
8. Add basic audit logging for master-data mutations.
9. Add frontend error handling and i18n mapping.

Definition of Done:

- User can log in.
- Protected routes block unauthenticated access.
- User can create/list/edit customers, suppliers, categories, products, and warehouses.
- Database changes persist across app restarts.
- Errors are returned as `AppError` and displayed safely in the UI.
- Audit records are captured for the vertical slice.

### Phase 2: Inventory core

Goal: make stock reliable before invoices depend on it.

Tasks:

1. Add stock movement schema and commands.
2. Add stock balance queries.
3. Add manual stock adjustment workflow with reason and audit log.
4. Add product movement report.
5. Add low-stock/reorder fields where needed.
6. Add purchase and sale price history commands.

Definition of Done:

- User can view current stock by product and warehouse.
- User can inspect all movements for a product.
- User can perform audited stock adjustments.
- Price changes are recorded in history.
- Inventory values use approved money and measurement strategies.

### Phase 3: Sales and purchase invoices

Goal: implement business-critical invoice workflows with automatic stock impact.

Tasks:

1. Implement sales invoice draft/create/update workflows.
2. Implement sales invoice approval/cancel/return workflow.
3. Implement purchase invoice draft/create/update workflows.
4. Implement purchase invoice approval/cancel/return workflow.
5. Ensure invoice approval creates stock movements transactionally.
6. Add tax, discount, rounding, and measurement calculation rules.
7. Add company settings and logo handling for invoices.
8. Generate PDF/print output according to selected Arabic strategy.

Definition of Done:

- User can create and approve sales invoices.
- User can create and approve purchase invoices.
- Approved invoices update inventory correctly.
- Cancel/return flows do not delete historical approved documents.
- Invoice totals are calculated using approved precision rules.
- Invoice PDF/print output handles Arabic and English correctly.

### Phase 4: Accounts, payments, and ledger

Goal: make customer and supplier balances traceable and auditable.

Tasks:

1. Add ledger/account transaction schema.
2. Add receipt workflow for customer payments.
3. Add payment workflow for supplier payments.
4. Link invoice approval and payments to ledger transactions.
5. Add customer account statement.
6. Add supplier account statement.
7. Add audited manual account adjustment workflow.

Definition of Done:

- Customer balance is reproducible from ledger rows.
- Supplier balance is reproducible from ledger rows.
- Payments and receipts affect statements correctly.
- Manual adjustments require a reason and audit log.

### Phase 5: Reports, backup, and polish

Goal: add operational visibility and production readiness.

Tasks:

1. Inventory report.
2. Sales report.
3. Purchase report.
4. Profit report.
5. Price changes report.
6. Audit log viewer.
7. Permission matrix UI.
8. Backup/export/import strategy.
9. Dashboard and charts.
10. Additional tests and quality gates.

Definition of Done:

- Users can review stock, sales, purchases, profit, and account balances.
- Admins can inspect audit history.
- Backup/export/import story is documented and tested.
- Reports match invoice, stock movement, and ledger sources of truth.

---

## Security Notes

### CSP

Do not leave production `tauri.conf.json` with:

```json
"csp": null
```

Define a restrictive CSP before release. During development, document any temporary relaxations.

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
- Treat stock movements and ledger transactions as sources of truth.

### Do Not

- Do not use `REAL`, `f64`, or JavaScript floating-point values as persisted money.
- Do not call `unwrap()` or `expect()` inside Tauri commands for recoverable errors.
- Do not expose raw SQL errors to users.
- Do not run a full init SQL file on every startup as a migration strategy.
- Do not store secrets in plaintext files or generic app settings.
- Do not leave production CSP as `null`.
- Do not call `invoke` directly from React components.
- Do not add broad Tauri filesystem permissions without a scoped need.
- Do not delete approved invoices to correct mistakes; use cancel, return, or adjustment workflows.

---

## Recommended Next Specification

Use Spec Kit to define the next product-level feature before adding more domain tables:

```text
Inventory and Accounts Core for Steel, Sheet Metal, and Accessories
```

The specification should cover:

- customers and suppliers
- product categories, units, and warehouse setup
- steel/sheet/accessory product attributes
- stock movements and stock balance rules
- sales and purchase invoice lifecycle
- customer/supplier ledger rules
- payments and receipts
- price history
- Arabic/English invoice output constraints
- MVP reports

Do not start implementation of inventory, purchase invoices, or ledger tables until this specification, plan, and task breakdown are complete.
