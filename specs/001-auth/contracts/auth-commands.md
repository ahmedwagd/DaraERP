# Tauri Command Contracts — 001-auth

## Backend Commands

All commands return `Result<T, AppError>`. Error codes are from the project error convention.

### `login`

```
Command: login
Input:   { email: string, password: string }
Output:  { user: { id: string, name: string, email: string, role: string, language: string } }
```

**Behavior**:
1. Validate email format and password non-empty.
2. Look up user by email; verify `is_active = 1`.
3. Verify password against stored argon2 hash.
4. Generate 32-byte random refresh token → SHA-256 hash → store in `refresh_tokens` with new `family_id`.
5. Generate JWT access token (user_id, role, language, exp=now+15min, iat=now).
6. Store refresh token (raw) in OS keychain under account `refresh-token`.
7. Set `AppState.session = Some(Session{...})`.
8. Write `LOGIN` audit event.
9. Return user info.

**Error codes**: `NOT_FOUND`, `AUTH_INVALID_CREDENTIALS`, `DATABASE_ERROR`, `IO_ERROR`.

### `logout`

```
Command: logout
Input:   {} (no args — uses current session from AppState)
Output:  null
```

**Behavior**:
1. Read current session from AppState.
2. Revoke all refresh tokens for user's active family (DELETE or UPDATE is_revoked=1).
3. Clear `refresh-token` entry from OS keychain.
4. Set `AppState.session = None`.
5. Write `LOGOUT` audit event.
6. Return null.

**Error codes**: `DATABASE_ERROR`, `IO_ERROR`.

**Partial failure handling**: If DB revoke succeeds but keychain clear fails, session is still set to None (credentials are invalidated in DB). Log the keychain error. The stale keychain entry will be ignored on next launch (DB check fails).

### `refresh_session`

```
Command: refresh_session
Input:   {} (no args — reads refresh token from keychain)
Output:  { user: { id: string, name: string, email: string, role: string, language: string } }
```

**Behavior**:
1. Read `refresh-token` from OS keychain. If absent or unreadable, return error.
2. Hash the token (SHA-256) → look up in `refresh_tokens` table.
3. Verify: token exists, `is_revoked = 0`, `expires_at > now`.
4. Check for theft: if this token's predecessor in the same family was already used (is_revoked=1), revoke entire family, write `TOKEN_THEFT_DETECTED` audit event, return error.
5. Rotation: mark current token `is_revoked = 1`. Generate new random token → store new row with same `family_id`.
6. Store new refresh token in keychain.
7. Generate new JWT access token.
8. Set `AppState.session = Some(Session{...})`.
9. Return user info.

**Error codes**: `AUTH_INVALID_CREDENTIALS`, `AUTH_EXPIRED_TOKEN`, `DATABASE_ERROR`, `IO_ERROR`.

### `get_current_user`

```
Command: get_current_user
Input:   {} (no args — uses current session)
Output:  { user: { id: string, name: string, email: string, role: string, language: string } } | null
```

**Behavior**:
1. Read AppState.session. If None, return null.
2. Look up user from DB by user_id. Verify `is_active = 1`.
3. Return user info.

**Error codes**: `DATABASE_ERROR`.

## Frontend Service Types

These match the backend command contracts above. TypeScript types for the wrapper layer:

```typescript
interface User {
  id: string;
  name: string;
  email: string;
  role: "ADMIN" | "MANAGER" | "USER";
  language: "en" | "ar";
}

interface LoginArgs {
  email: string;
  password: string;
}

interface LoginResponse {
  user: User;
}

// logout: no args, returns null

interface RefreshSessionResponse {
  user: User;
}

interface GetCurrentUserResponse {
  user: User;
}
```

## Frontend Service (src/services/auth.ts)

```typescript
import { invokeCommand } from "../lib/tauri";
import type { User, LoginArgs, LoginResponse, RefreshSessionResponse, GetCurrentUserResponse } from "./types";

export async function login(email: string, password: string): Promise<User> {
  const res = await invokeCommand<LoginResponse, LoginArgs>("login", { email, password });
  return res.user;
}

export async function logout(): Promise<void> {
  await invokeCommand("logout");
}

export async function refreshSession(): Promise<User> {
  const res = await invokeCommand<RefreshSessionResponse>("refresh_session");
  return res.user;
}

export async function getCurrentUser(): Promise<User | null> {
  const res = await invokeCommand<GetCurrentUserResponse>("get_current_user");
  return res?.user ?? null;
}
```
