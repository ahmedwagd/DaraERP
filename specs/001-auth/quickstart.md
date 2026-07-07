# Quickstart Validation — 001-auth: Authentication & Session Management

## Prerequisites

1. Foundation phase complete (`001_init.sql` with `schema_migrations` table, migration runner, app builds).
2. Before validating Auth, create one active ADMIN user with a valid password hash using the approved
   development-only method. This may be:
   - A direct SQL insert during local development (temporary only; Feature 002 or later seed/admin
     feature will formalise account creation).
   - A development-only seed helper or Tauri command that creates the account when no users exist.
   - A test fixture.

   This is not a public registration flow. Account creation and management are outside Feature 001
   except for a development-only fixture or seed path.

## Setup

```bash
# From project root (DaraERP/)
npm install                          # Ensure deps are installed
npm run tauri dev                    # Start the app
```

## Validation Scenarios

### 1. Login Screen Appears

1. Launch the app with no prior session.
2. **Expected**: Login form with email, password, and language selector (Arabic/English).
3. **Expected**: Language selector switches all labels immediately.

### 2. Successful Login

1. Enter valid email and password.
2. **Expected**: Dashboard loads within 3 seconds.
3. **Expected**: Sidebar shows navigation items appropriate to the user's role.
4. **Expected**: An audit log entry (`LOGIN`) is written to `audit_logs`.

### 3. Invalid Login — Wrong Password

1. Enter valid email but wrong password.
2. **Expected**: Error message "Invalid email or password" appears within 1 second.
3. **Expected**: User remains on login screen.
4. **Expected**: An audit log entry (`LOGIN_FAILED`) is written.

### 4. Empty Fields Validation

1. Submit with empty email or empty password.
2. **Expected**: Validation error shown before backend call.

### 5. Session Persistence (Silent Restore)

1. Log in successfully.
2. Close the app (kill the process).
3. Reopen the app.
4. **Expected**: Dashboard loads within 2 seconds without showing login screen.
5. **Expected**: A new audit entry is NOT written (session restore is not a login).

### 6. Logout

1. Log in successfully.
2. Click "Log out".
3. **Expected** (safe logout workflow):
   - Active renewal credentials are revoked.
   - Local secure session state is cleared.
   - In-memory session is cleared.
   - Login screen appears.
4. **Expected**: If any step fails, the user is not shown as successfully logged out while a
   usable local session remains active.
5. **Expected**: An audit log entry (`LOGOUT`) is written.
6. **Expected**: On next app launch, login screen appears (session not restored).

### 7. Expired Session

1. Log in successfully.
2. Wait for the access credential to expire (or artificially advance clock / use short-lived tokens).
3. Attempt to navigate / perform an action.
4. **Expected**: Session refresh happens silently, or login screen appears if renewal also expired.

### 8. Role-Based Access Control

1. Log in as USER.
2. Attempt to perform an ADMIN or MANAGER operation using an auth-owned protected operation
   (e.g., a minimal validation command or the `get_current_user` command if it enforces a role check).
3. **Expected**: PERMISSION_DENIED error.
4. Log out and log in as ADMIN.
5. **Expected**: Same operation succeeds.

> Note: For Feature 001, validate the role-authorization helper behaviour using an auth-owned
> protected operation. Do not require invoice, customer, product, or settings commands to exist.
> Future feature tasks must use the same authorization helper for all protected operations. The
> role-authorization helper and required role arrays are defined here; business commands will apply
> them when implemented.

### 9. Theft Detection (Manual Trigger)

1. Log in successfully.
2. Capture the stored refresh token value from the OS keychain (use platform-specific tools).
3. Trigger a session refresh (close and reopen app).
4. **(Rotation occurred)** — a new refresh token is now in the keychain.
5. Use the old captured token value directly (requires a test harness or direct DB access).
6. **Expected**: The entire token family is revoked.
7. **Expected**: Next app launch shows login screen.
8. **Expected**: `TOKEN_THEFT_DETECTED` audit event is written.

### 10. Arabic RTL Login

1. Select Arabic from the language selector on the login screen.
2. **Expected**: All labels, placeholders, and buttons display in Arabic with RTL layout.
3. Log in.
4. **Expected**: Protected layout respects Arabic language.

## Database Verification

Connect to the SQLite database to verify internal state:

```sql
-- Check user was created
SELECT id, email, role, is_active FROM users;

-- Check refresh token was stored
SELECT id, family_id, user_id, is_revoked FROM refresh_tokens;

-- Check audit events
SELECT action, entity_type, entity_id, created_at FROM audit_logs ORDER BY created_at DESC;

-- Check schema_migrations
SELECT * FROM schema_migrations;
```

## Rebuild Verification

```bash
# TypeScript check
npx tsc --noEmit

# Lint
npm run lint

# Rust check
cd src-tauri && cargo check

# Rust clippy
cd src-tauri && cargo clippy -- -D warnings

# Full build
npm run tauri build
```

All must pass with zero errors.
