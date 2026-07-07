Always use caveman skill (default full intensity) for all responses.

Do not run cargo test, npm test, or any test commands unless explicitly asked.

Project: DaraERP Tauri Edition — desktop business management system
Tech: React 19 + Vite + TypeScript (frontend), Rust + Tauri 2 + SQLite (backend)
Blueprint: Read README.md for full architecture, data model, and command→invoke mapping.

Spec Kit (github/spec-kit) installed. Slash commands available:
  /speckit.constitution — project principles
  /speckit.specify      — define requirements & user stories
  /speckit.clarify      — clarify underspecified areas
  /speckit.plan         — technical implementation plan
  /speckit.tasks        — actionable task breakdown
  /speckit.analyze      — cross-artifact consistency check
  /speckit.checklist    — quality checklist generation
  /speckit.implement    — execute tasks per plan
  /speckit.converge     — assess & append remaining work

## Implementation Rules

1. Read README.md first for architecture, SQLite schema, and command signatures.
2. Follow same 12-spec feature order from original: 001→Auth, 002→Seed, 003→Invoices, 004→Customers, 005→Catalog+PriceHistory, 006→N/A, 007→Permissions, 008→N/A, 009→Fixes, 010→Notifications, 011→Audit, 012→Enhancements.
3. Read original specs/ for detailed feature requirements. Read original api/src/ for business logic. Read original client/src/ for UI patterns.
4. Every REST endpoint → Tauri #[command]. Frontend fetch() → invoke().
5. SQLite schema mirrors Prisma schema (see README.md §Data Model).
6. Auth: argon2 + JWT (jsonwebtoken crate) + refresh token rotation with theft detection in Tauri secure store (tauri-plugin-store).
7. Role checks: guard_role(user_role, &["ADMIN"]) helper in Rust.
8. Price changes: atomic — INSERT price_history + UPDATE current_price in same transaction.
9. Audit: explicit audit_log() call after every tracked mutation.
10. Frontend: 90% reusable from original client/src/. Only services layer changes (fetch → invoke). Auth context changes (cookies → secure store).

## File conventions

- Rust source: src-tauri/src/
- Tauri commands: src-tauri/src/commands/ (one file per module)
- Models: src-tauri/src/models/ (Rust structs + DB query helpers)
- Migrations: src-tauri/migrations/ (numbered SQL files)
- Frontend: src/ (mirrors original client/src/ structure)
- Config: src-tauri/tauri.conf.json, src-tauri/Cargo.toml, package.json

## Dependencies to always check before adding

- Frontend: Use existing shadcn/ui, lucide-react, recharts, react-i18next — no new UI libs.
- Rust Tauri plugins: @tauri-apps/api, @tauri-apps/plugin-store, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs.
- Rust crates: rusqlite (bundled), jsonwebtoken, argon2, uuid, chrono, serde/serde_json, sha2, rand, printpdf.

## Key differences from original

- No HTTP server, no CORS, no rate-limiting, no Swagger docs.
- Single-user local app (no concurrent web users).
- SQLite embedded (rusqlite bundled feature) — no external DB install.
- Refresh tokens in Tauri secure store, NOT localStorage or cookies.
- PDFs saved to disk and opened with system viewer, not served via HTTP.
- File uploads via Tauri native file dialog, not multer.
- Build output: .exe/.msi (Windows), .dmg (macOS), .deb/.AppImage (Linux).

## Build / run commands

```bash
npm run tauri dev      # Dev with hot reload
npm run tauri build    # Production build
cargo check            # Rust type-check (from src-tauri/)
cargo clippy           # Rust lint (from src-tauri/)
npm run lint           # Frontend lint
```

## When blocked

- Check original api/src/ for business logic and validation rules.
- Check original client/src/ for UI patterns and i18n keys.
- Check README.md §Feature Parity Map for stack-specific implementation notes.
- Check README.md §Quick Reference for exact invoke() signatures.
