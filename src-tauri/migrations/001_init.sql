-- DaraERP — Initial Schema (v1)
-- Foundation Fix (2026-07-07): All monetary columns changed from REAL → INTEGER minor units.
-- Tax rates changed from REAL → INTEGER basis points.
-- Renames:
--   products.current_price       → current_price_minor       (REAL → INTEGER)
--   price_history.price          → price_minor               (REAL → INTEGER)
--   invoices.subtotal            → subtotal_minor            (REAL → INTEGER)
--   invoices.discount_total      → discount_minor            (REAL DEFAULT 0 → INTEGER DEFAULT 0)
--   invoices.tax_rate            → tax_rate_bps              (REAL → INTEGER)
--   invoices.tax_total           → tax_total_minor           (REAL → INTEGER)
--   invoices.grand_total         → grand_total_minor         (REAL → INTEGER)
--   invoice_items.unit_price     → unit_price_minor          (REAL → INTEGER)
--   invoice_items.discount       → discount_minor            (REAL → INTEGER)
--   invoice_items.tax_rate       → tax_rate_bps              (REAL → INTEGER)
--   invoice_items.line_total     → line_total_minor          (REAL → INTEGER)
-- Rationale: Constitution Principle I — Financial Precision (Non-Negotiable).
--   All monetary values MUST be stored and computed as integer minor units.
--   Floating-point types MUST NOT appear in any persisted financial column.

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     TEXT PRIMARY KEY,
    applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY,
    email           TEXT NOT NULL UNIQUE,
    name            TEXT,
    avatar_url      TEXT,
    password_hash   TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'USER',
    language        TEXT NOT NULL DEFAULT 'en',
    department      TEXT,
    team            TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id          TEXT PRIMARY KEY,
    token_hash  TEXT NOT NULL,
    family_id   TEXT NOT NULL,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TEXT NOT NULL,
    is_revoked  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);

CREATE TABLE IF NOT EXISTS customers (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    email       TEXT,
    phone       TEXT,
    address     TEXT,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS products (
    id                          TEXT PRIMARY KEY,
    category_id                 TEXT NOT NULL REFERENCES categories(id),
    sku                         TEXT NOT NULL UNIQUE,
    name                        TEXT NOT NULL,
    thickness                   REAL,
    weight                      REAL,
    unit                        TEXT,
    notes                       TEXT,
    current_price_minor         INTEGER,
    current_price_effective_date TEXT,
    is_active                   INTEGER NOT NULL DEFAULT 1,
    created_at                  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);

CREATE TABLE IF NOT EXISTS price_history (
    id                TEXT PRIMARY KEY,
    product_id        TEXT NOT NULL REFERENCES products(id),
    price_minor       INTEGER NOT NULL,
    effective_date    TEXT NOT NULL,
    change_reason     TEXT,
    changed_by_user_id TEXT REFERENCES users(id),
    created_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_price_history_product_date ON price_history(product_id, effective_date);

CREATE TABLE IF NOT EXISTS invoices (
    id                TEXT PRIMARY KEY,
    invoice_number    INTEGER NOT NULL UNIQUE,
    customer_id       TEXT NOT NULL REFERENCES customers(id),
    subtotal_minor    INTEGER NOT NULL,
    discount_minor    INTEGER NOT NULL DEFAULT 0,
    tax_rate_bps      INTEGER NOT NULL DEFAULT 0,
    tax_total_minor   INTEGER NOT NULL DEFAULT 0,
    grand_total_minor INTEGER NOT NULL,
    currency          TEXT NOT NULL DEFAULT 'EGP',
    status            TEXT NOT NULL DEFAULT 'DRAFT',
    due_date          TEXT NOT NULL,
    notes             TEXT,
    is_active         INTEGER NOT NULL DEFAULT 1,
    created_by_id     TEXT NOT NULL REFERENCES users(id),
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_invoices_customer ON invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_invoices_created_by ON invoices(created_by_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);

CREATE TABLE IF NOT EXISTS invoice_items (
    id                TEXT PRIMARY KEY,
    invoice_id        TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    product_id        TEXT NOT NULL REFERENCES products(id),
    quantity          INTEGER NOT NULL DEFAULT 1,
    unit_price_minor  INTEGER NOT NULL,
    discount_minor    INTEGER NOT NULL DEFAULT 0,
    tax_rate_bps      INTEGER NOT NULL DEFAULT 0,
    line_total_minor  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice ON invoice_items(invoice_id);

CREATE TABLE IF NOT EXISTS notifications (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type        TEXT NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    entity_type TEXT,
    entity_id   TEXT,
    is_read     INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_notifications_user_read ON notifications(user_id, is_read);
CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at);

CREATE TABLE IF NOT EXISTS audit_logs (
    id          TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    action      TEXT NOT NULL,
    changes     TEXT,
    user_id     TEXT NOT NULL,
    user_email  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_type);
CREATE INDEX IF NOT EXISTS idx_audit_entity_id ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);

CREATE TABLE IF NOT EXISTS company_settings (
    id           TEXT PRIMARY KEY DEFAULT 'default',
    company_name TEXT NOT NULL DEFAULT 'DaraERP',
    logo_path    TEXT,
    phone        TEXT,
    email        TEXT,
    website      TEXT,
    tax_id       TEXT,
    address      TEXT,
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
