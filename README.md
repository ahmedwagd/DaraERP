# DaraERP

DaraERP is a desktop ERP and invoicing system built with **Tauri**, **Rust**, **SQLite**, **React**, and **TypeScript**. The project is a migration from an earlier web-based NestJS/Prisma architecture into a local-first desktop application.

The goal is to provide a fast, reliable, bilingual ERP system with strong support for invoices, customers, products, company settings, permissions, audit logs, notifications, and Arabic-ready document generation.

---

## Current Status

This repository is currently in the **foundation/scaffold stage**.

The architecture blueprint is defined, but most implementation modules are not yet complete. The current priority is to stabilize project structure, developer instructions, financial precision rules, PDF strategy, security configuration, and migration handling before expanding feature work.

---

## Technology Stack

### Desktop Runtime

- Tauri v2
- Rust
- SQLite
- rusqlite

### Frontend

- React
- TypeScript
- Vite
- Tauri JavaScript API

### Planned Frontend Libraries

- React Router
- i18next / react-i18next
- Tailwind CSS
- shadcn/ui and Radix primitives
- Recharts
- Lucide React
- Sonner

### Planned Backend Capabilities

- Authentication and session handling
- SQLite migrations
- Seed data
- Tauri command layer
- Audit logging
- Invoice PDF generation
- Notifications engine
- Permission checks

---

## Repository Structure

Expected structure:

```text
.
├── README.md                  # Main project / architecture documentation
├── AGENTS.md                  # Agent and development instructions
├── DaraERP/
│   ├── README.md              # App-specific setup and implementation guide
│   ├── package.json
│   ├── src/
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   ├── index.css
│   │   ├── lib/
│   │   ├── shared/
│   │   ├── features/
│   │   ├── components/
│   │   └── i18n/
│   └── src-tauri/
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       └── src/
│           ├── lib.rs
│           ├── auth/
│           ├── commands/
│           ├── models/
│           ├── migrations/
│           ├── seed.rs
│           ├── audit.rs
│           ├── pdf.rs
│           └── notifications_engine.rs
```

---

## Important Architecture Decisions

### 1. Financial Precision

Do **not** use `f64` for persisted financial values.

DaraERP should use one of the following approaches:

1. Store monetary values as integer minor units, such as cents or piasters.
2. Use a decimal type such as `rust_decimal` for calculation-heavy financial logic.

Invoice totals, taxes, discounts, payments, balances, and price history must be deterministic and rounding-safe.

### 2. Arabic PDF Generation

Arabic invoice generation must be decided before invoice templates are implemented.

Arabic PDFs require:

- Right-to-left layout
- Arabic glyph shaping
- Font embedding
- Correct line wrapping
- Consistent print output

Recommended strategies:

1. HTML/CSS invoice template rendered through a browser/WebView-style pipeline.
2. Rust-native PDF generation with proper Arabic shaping support.
3. Frontend-generated PDF only if layout, fonts, and print fidelity are validated.

### 3. Database Migrations

A single `001_init.sql` loaded on every startup is not enough for long-term schema evolution.

The project should include a migration runner that tracks applied migrations in a dedicated table, for example:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);
```

Each migration should run once, inside a transaction.

### 4. Security Baseline

Before production builds, the project should define:

- A restrictive Tauri Content Security Policy instead of `csp: null`
- Least-privilege Tauri v2 plugin permissions/capabilities
- A clear token/session storage policy
- SQLite file location and backup policy
- Secret management rules
- Filesystem access boundaries

Sensitive tokens should not be stored in plain persistent key-value storage.

---

## Setup

### Prerequisites

Install:

- Node.js LTS
- npm
- Rust stable
- Tauri system dependencies for your operating system

### Install Dependencies

From the app directory:

```bash
cd DaraERP
npm install
```

### Run Development App

```bash
npm run tauri dev
```

### Build Frontend

```bash
npm run build
```

### Build Desktop App

```bash
npm run tauri build
```

### Rust Check

```bash
cd src-tauri
cargo check
```

---

## Recommended Scripts

The app should eventually expose these npm scripts:

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

If `npm run lint` is referenced in `AGENTS.md`, it must exist in `package.json`.

---

## Backend Development Pattern

Tauri commands should follow a consistent pattern:

```rust
#[tauri::command]
pub async fn example_command() -> Result<ExampleResponse, String> {
    // Validate input
    // Run business logic
    // Return typed response or mapped error
    Ok(ExampleResponse {})
}
```

Recommended backend modules:

```text
src-tauri/src/
├── auth/
│   ├── mod.rs
│   ├── password.rs
│   ├── session.rs
│   └── guard.rs
├── commands/
│   ├── mod.rs
│   ├── auth.rs
│   ├── customers.rs
│   ├── products.rs
│   ├── invoices.rs
│   └── settings.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   ├── customer.rs
│   ├── product.rs
│   └── invoice.rs
├── migrations/
├── seed.rs
├── audit.rs
├── pdf.rs
└── notifications_engine.rs
```

All commands registered in Rust must be added to the Tauri invoke handler.

Example:

```rust
.invoke_handler(tauri::generate_handler![
    commands::auth::login,
    commands::auth::logout,
    commands::customers::list_customers,
])
```

---

## Frontend Development Pattern

Recommended frontend structure:

```text
src/
├── App.tsx
├── main.tsx
├── index.css
├── lib/
│   ├── tauri.ts
│   ├── format.ts
│   └── money.ts
├── shared/
│   ├── types/
│   ├── hooks/
│   └── utils/
├── features/
│   ├── auth/
│   ├── customers/
│   ├── products/
│   ├── invoices/
│   ├── settings/
│   └── audit/
├── components/
│   ├── ui/
│   └── layout/
└── i18n/
    ├── index.ts
    ├── en.json
    └── ar.json
```

Frontend calls to Rust should go through a typed wrapper instead of calling `invoke` directly everywhere.

Example:

```ts
import { invoke } from "@tauri-apps/api/core";

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}
```

---

## Implementation Roadmap

### Phase 0: Foundation

- Resolve README and AGENTS.md path mismatches
- Replace default Tauri README with project-specific documentation
- Fix missing or dead references to unavailable resources
- Decide financial precision model
- Decide Arabic PDF generation strategy
- Add migration runner
- Add minimal lint/build/check scripts
- Define Tauri CSP and plugin permission policy
- Define backend error-handling convention
- Decide desktop auth/session model

### Phase 1: Minimal Working Vertical Slice

- SQLite migration runner
- Seed data
- Login command
- Frontend login page
- Protected app shell
- Typed Tauri invoke wrapper
- One complete CRUD module, preferably customers
- Basic audit logging pattern

### Phase 2: Core ERP Features

- Customers
- Categories
- Products
- Price history
- Invoice creation and editing
- Invoice finalization
- Tax, discount, and rounding rules
- Invoice PDF generation
- Company settings and logo upload

### Phase 3: Supporting Features

- Notifications engine
- Notification bell UI
- Permission matrix
- Audit log viewer
- Reports and dashboard
- Backup/export flow

---

## Testing Strategy

Recommended test layers:

### Rust

- Unit tests for money calculations
- Unit tests for auth/password/session logic
- Migration tests
- Command-level integration tests where practical

### Frontend

- Component tests for critical forms
- Utility tests for formatting and money display
- Basic route/auth flow tests

### Manual QA

Before release, verify:

- Arabic and English UI
- RTL layout
- Invoice totals and rounding
- PDF rendering and printing
- Database migration from older versions
- Backup and restore behavior
- Permission boundaries

---

## Development Rules

- Do not expand features before core architectural decisions are resolved.
- Do not persist money as floating-point values.
- Do not leave production CSP disabled.
- Do not add broad filesystem permissions without a clear need.
- Do not call Tauri commands directly from scattered UI components; use a typed service layer.
- Do not register empty command modules as if they are implemented.
- Keep business logic in Rust/backend modules where consistency and auditability matter.
- Keep frontend focused on interaction, display, validation, and user workflow.

---

## First Implementation Target

The first target should be a complete vertical slice:

```text
Login → Protected Layout → Customers CRUD → Audit Entry → SQLite Persistence
```

This creates a repeatable pattern for the rest of the ERP modules.

---

## License

To be defined.
