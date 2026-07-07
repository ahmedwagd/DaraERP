# Feature Specification: User Management

**Feature Branch**: `002-user-management`

**Created**: 2026-07-07

**Status**: Draft

**Input**: DaraERP User Management (Feature 002): production-ready administrator-only user management
commands (`create_user`, `list_users`, `get_user`, `update_user`, `set_user_active`) replacing
reliance on the debug-only seed fixture for normal account creation, while keeping the debug seed
available only as a development convenience. Backend commands plus typed frontend service wrappers
only; no user management UI pages in this feature.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Create User (Priority: P1)

An ADMIN opens the app and creates a new user account by providing email, name, password, role, and
language. The system hashes the password, inserts an active user record, writes an audit event, and
returns a safe user response without password secrets.

**Why this priority**: Creating users is the prerequisite for multi-user operation. The debug seed
creates one admin; this story enables creation of regular users, managers, and additional admins.

**Independent Test**: Log in as ADMIN → call `create_user` with valid inputs → receive `UserResponse`.
Log in as MANAGER → call `create_user` → receive `PERMISSION_DENIED`.

**Acceptance Scenarios**:

1. **Given** an active ADMIN session, **When** `create_user` is called with valid email, name,
   password, valid role, and valid language, **Then** the user is inserted as active, the password
   is hashed, a `USER_CREATED` audit event is written, and a `UserResponse` is returned without
   `password_hash`.
2. **Given** an existing email, **When** `create_user` is called with a duplicate email, **Then** a
   `CONFLICT` error is returned and no user is created.
3. **Given** an invalid role, invalid language, empty name, empty password, or invalid email format,
   **When** `create_user` is called, **Then** a `VALIDATION_ERROR` is returned.
4. **Given** a MANAGER or USER session, **When** `create_user` is called, **Then**
   `PERMISSION_DENIED` is returned.
5. **Given** no active session, **When** `create_user` is called, **Then** `UNAUTHORIZED` is returned.

---

### User Story 2 — View Users (Priority: P1)

An ADMIN lists all users with optional role and active-status filters, and fetches a single user by
ID. Responses never include password hashes.

**Why this priority**: Administrators must be able to see who exists in the system before they can
manage those accounts.

**Independent Test**: Log in as ADMIN → call `list_users` → receive list of `UserResponse` items.
Call `get_user` with a valid ID → receive the user. Call `get_user` with a nonexistent ID → receive
`NOT_FOUND`.

**Acceptance Scenarios**:

1. **Given** an active ADMIN session, **When** `list_users` is called, **Then** a list of
   `UserResponse` items is returned without `password_hash`.
2. **Given** an optional `role` filter, **When** `list_users` is called with the filter, **Then**
   only users with the matching role are returned.
3. **Given** an optional `is_active` filter, **When** `list_users` is called with the filter,
   **Then** only users with the matching active status are returned.
4. **Given** a valid user ID, **When** `get_user` is called, **Then** the `UserResponse` is returned
   without `password_hash`.
5. **Given** a nonexistent user ID, **When** `get_user` is called, **Then** `NOT_FOUND` is returned.

---

### User Story 3 — Update User (Priority: P2)

An ADMIN updates an existing user's email, name, role, or language. Only provided fields are changed.
`is_active` is never modified by this operation. The last active ADMIN cannot be demoted.

**Why this priority**: User profiles change over time. Admins need to correct names, update email
addresses, reassign roles, and change language preferences.

**Independent Test**: Call `update_user` with a new name → user reflects the change. Call
`update_user` with no fields → user is unchanged. Attempt to demote the last ADMIN → `BAD_REQUEST`.

**Acceptance Scenarios**:

1. **Given** a valid user ID and one or more optional fields, **When** `update_user` is called,
   **Then** only the provided fields are updated, `updated_at` is refreshed, a `USER_UPDATED`
   audit event is written, and the updated `UserResponse` is returned.
2. **Given** an invalid email, invalid role, or invalid language, **When** `update_user` is called,
   **Then** `VALIDATION_ERROR` is returned.
3. **Given** a new email that already belongs to another user, **When** `update_user` is called,
   **Then** `CONFLICT` is returned.
4. **Given** an attempt to change an active ADMIN's role to non-ADMIN, and that user is the last
   active ADMIN, **When** `update_user` is called, **Then** `BAD_REQUEST` is returned.
5. **Given** an update attempt, **When** `update_user` is called, **Then** `is_active` is NOT changed
   by this operation.

---

### User Story 4 — Activate / Deactivate User (Priority: P2)

An ADMIN activates or deactivates a user. Deactivation revokes all refresh tokens for that user. An
ADMIN cannot deactivate themselves. The last active ADMIN cannot be deactivated.

**Why this priority**: Account lifecycle management. Deactivating a user immediately terminates their
existing sessions.

**Independent Test**: Call `set_user_active` with `is_active: false` → user is deactivated and
refresh tokens are revoked. Call with `is_active: true` → user is reactivated.

**Acceptance Scenarios**:

1. **Given** an active user who is not the last ADMIN, **When** `set_user_active` is called with
   `is_active: false`, **Then** the user is deactivated, all refresh tokens for that user are
   revoked in the same transaction, a `USER_DEACTIVATED` audit event is written, and the updated
   `UserResponse` is returned.
2. **Given** an inactive user, **When** `set_user_active` is called with `is_active: true`, **Then**
   the user is reactivated, a `USER_ACTIVATED` audit event is written, and the updated
   `UserResponse` is returned.
3. **Given** the acting ADMIN is the target, **When** `set_user_active` is called, **Then**
   `BAD_REQUEST` is returned (self-deactivation blocked).
4. **Given** the target is the last active ADMIN, **When** `set_user_active` is called with
   `is_active: false`, **Then** `BAD_REQUEST` is returned.
5. **Given** a deactivated user with existing refresh tokens, **When** a session refresh is
   attempted, **Then** the refresh fails and the user is forced to re-authenticate.

---

### Edge Cases

- **Duplicate email**: Creating a user or updating email to one that already exists returns
  `CONFLICT`. The constraint is enforced at the database level and mapped to the project error code.
- **Last active ADMIN demotion**: If an ADMIN's role is changed to non-ADMIN and they are the only
  remaining active ADMIN, the operation returns `BAD_REQUEST`. The system must always have at least
  one active ADMIN.
- **Last active ADMIN deactivation**: Same guard as demotion. The last active ADMIN cannot be
  deactivated.
- **Self-deactivation**: An ADMIN cannot deactivate their own account. Returns `BAD_REQUEST`.
- **Deactivating an already inactive user**: The operation succeeds as a no-op; the user remains
  inactive. An audit event is still written.
- **Activating an already active user**: The operation succeeds as a no-op; the user remains active.
  An audit event is still written.
- **Updating with no fields provided**: Updating with no fields provided: update_user returns VALIDATION_ERROR because no meaningful change was requested.
- **Invalid role or language filter on `list_users`**: Invalid role filter values return VALIDATION_ERROR. Unknown but valid filters that match no users return an empty list.
- **Deactivated user attempts session refresh**: Their refresh tokens were revoked, so the refresh
  fails and the app shows the login screen.
- **Audit write failure during mutation**: The mutation and audit run inside one transaction. If
  the audit insert fails, the entire transaction rolls back and a `DATABASE_ERROR` is returned.
- **Database connection lost mid-operation**: Transaction rollback ensures no partial state.
  Returns `DATABASE_ERROR`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: All user management commands MUST require an active authenticated ADMIN session.
- **FR-002**: Non-ADMIN access MUST return `PERMISSION_DENIED`.
- **FR-003**: Missing or invalid session MUST return `UNAUTHORIZED` or the project-standard auth
  error.
- **FR-004**: `create_user` MUST accept email, name, password, role, and language and create an
  active user account.
- **FR-005**: System MUST validate email format, non-empty name, non-empty password, valid role
  (ADMIN | MANAGER | USER), and valid language (en | ar) before creation.
- **FR-006**: System MUST hash passwords using the existing `auth::hash_password` function. Passwords
  MUST NOT be stored in plaintext.
- **FR-007**: System MUST never return `password_hash` or raw secrets in any user management response.
- **FR-008**: `list_users` MUST return a list of users with optional `role` and `is_active` filters.
- **FR-009**: `get_user` MUST fetch a user by ID or return `NOT_FOUND`.
- **FR-010**: `update_user` MUST update only provided fields: email, name, role, language.
- **FR-011**: `update_user` MUST NOT modify `is_active`. Active status is changed only via
  `set_user_active`.
- **FR-012**: System MUST prevent demoting the last active ADMIN to a non-ADMIN role.
- **FR-013**: System MUST prevent an ADMIN from deactivating their own account.
- **FR-014**: System MUST prevent deactivating the last active ADMIN.
- **FR-015**: Deactivating a user MUST revoke all refresh tokens for that user in the same
  transaction.
- **FR-016**: Every user mutation (create, update, activate, deactivate) MUST write a corresponding
  audit event.
- **FR-017**: Operations combining user mutation, token revocation, and audit logging MUST run inside
  a single database transaction. Commit on full success; roll back on any error.
- **FR-018**: Duplicate email on creation or update MUST return `CONFLICT`.
- **FR-019**: Invalid input (role, language, empty fields, email format) MUST return
  `VALIDATION_ERROR`.
- **FR-020**: `set_user_active` MUST return the updated `UserResponse`.
- **FR-021**: update_user MUST return VALIDATION_ERROR when no update fields are provided.

### Key Entities

- **User**: Account identity stored in the `users` table. Fields: id (UUID), email (unique),
  name, password_hash (argon2id), role (ADMIN | MANAGER | USER), language (en | ar), is_active
  (boolean), created_at, updated_at. Does not include `username` — email is the login identifier.
- **UserResponse**: Safe public representation of a user. Fields: id, email, name, role, language,
  is_active, created_at, updated_at. Never includes password_hash.
- **Acting Admin**: The currently authenticated ADMIN user performing a management operation.
  Their identity is captured from the active session and recorded in audit events.
- **Audit Event**: A record written to the `audit_logs` table for every tracked user mutation.
  Includes acting admin ID/email, target user ID, action code, entity type "user", and a safe
  changed-field summary.
- **Refresh Token / Session Credential**: On deactivation, all refresh token families for the
  deactivated user are revoked (`is_revoked = 1`), terminating all existing sessions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An ADMIN can create a valid user and receive a safe `UserResponse` in the same
  request cycle.
- **SC-002**: A duplicate email returns `CONFLICT` without creating a second user record.
- **SC-003**: `list_users` returns correctly filtered results when role and/or active-status
  filters are provided.
- **SC-004**: The last active ADMIN cannot be demoted or deactivated; attempting either returns
  `BAD_REQUEST`.
- **SC-005**: Deactivating a user revokes all refresh tokens for that user in the same
  transaction as the status change.
- **SC-006**: 100% of user management commands are ADMIN-guarded (verified by calling each
  command from a non-ADMIN session and confirming `PERMISSION_DENIED`).
- **SC-007**: No user management response includes `password_hash`, raw password, or refresh
  token values.

## Audit Requirements

The following audit actions are defined for user management:

| Action | When |
|--------|------|
| `USER_CREATED` | A new user is created via `create_user` |
| `USER_UPDATED` | A user's email, name, role, or language is changed via `update_user` |
| `USER_ACTIVATED` | A user is activated via `set_user_active` |
| `USER_DEACTIVATED` | A user is deactivated via `set_user_active` |

Each audit event must include:

- Acting admin user ID and email (the ADMIN performing the operation).
- Target user ID (the user being created, updated, activated, or deactivated).
- Entity type `"user"`.
- Action code matching the operation.
- A safe changed-field summary (e.g., which fields were set on create, which were changed on update,
  the target role on activation/deactivation).

Passwords, password hashes, and raw tokens MUST NOT appear in audit `changes` JSON.

## Error Codes

| Code | Meaning | When |
|------|---------|------|
| `CONFLICT` | Duplicate unique value | Email already exists on create or update |
| `VALIDATION_ERROR` | Invalid input | Bad role, language, email format, or empty required fields |
| `PERMISSION_DENIED` | Insufficient role | Non-ADMIN user attempts a management command |
| `UNAUTHORIZED` | No active session | Missing or expired authenticated session |
| `NOT_FOUND` | Entity missing | User ID does not exist |
| `BAD_REQUEST` | Invalid operation | Self-deactivation, last-admin demotion, last-admin deactivation |
| `DATABASE_ERROR` | Internal DB failure | Sanitized database error (stack traces not exposed) |
| `INTERNAL_ERROR` | Unexpected failure | Lock errors, unexpected panics prevented by safe handling |

`BAD_REQUEST` covers self-deactivation, last-admin demotion, and last-admin deactivation. `CONFLICT`
covers duplicate email / unique constraint violations. Internal database errors are sanitized and
must not expose raw SQLite errors, stack traces, or paths to the frontend.

## Assumptions

- At least one active ADMIN account exists before these commands are tested, provided by the
  debug-only seed fixture (`admin@daraerp.local` / `admin123` for debug builds) or by a previously
  created ADMIN account.
- No self-registration flow in v1. Accounts are created exclusively by an ADMIN.
- No password reset flow in v1. Password changes are handled as an administrative process (future
  feature).
- No bulk user import/export in v1.
- No user groups, teams, or departments beyond the existing `department` and `team` columns
  (these columns exist in the schema but are not managed by Feature 002).
- The app is single-user desktop; concurrent ADMIN operations are improbable but the Mutex pattern
  protects against data races.
- `seed.rs` remains unchanged and only operates in `#[cfg(debug_assertions)]` builds.

## Planning Notes — Non-Normative

*This section is for the implementation plan (`/speckit.plan`), not stakeholder requirements.*

Technical constraints inherited from the project constitution and Feature 001:

- All user management commands follow the same Tauri command pattern established in Feature 001:
  validate → guard → lock DB → perform operation + audit in transaction → commit → return.
- Password hashing uses the existing `auth::hash_password` function (argon2id via `argon2` crate).
- Role enforcement uses the existing `auth::guard_role` helper.
- Session checking uses the existing `AppState.session` pattern.
- Audit logging uses the existing `audit::audit_log` function.
- Database queries follow the existing `models/mod.rs` pattern.
- No new Rust or frontend dependencies are required.
- `UserResponse` DTO lives in `commands/users.rs` (consistent with `UserResponse` in
  `commands/auth.rs`).
- Transaction wrapping follows the `rusqlite::Transaction` pattern established in Feature 001
  review fixes.
- `conn.transaction()` requires `&mut Connection`; callers lock the DB mutex as mutable.
