import { invokeCommand } from "../lib/tauri";

export interface UserResponse {
  id: string;
  email: string;
  name: string | null;
  role: string;
  language: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export async function createUser(
  email: string,
  name: string,
  password: string,
  role: string,
  language: string,
): Promise<UserResponse> {
  return invokeCommand<UserResponse>("create_user", {
    email,
    name,
    password,
    role,
    language,
  });
}

export async function listUsers(
  role?: string | null,
  isActive?: boolean | null,
): Promise<UserResponse[]> {
  return invokeCommand<UserResponse[]>("list_users", {
    role: role ?? null,
    is_active: isActive ?? null,
  });
}

export async function getUser(id: string): Promise<UserResponse> {
  return invokeCommand<UserResponse>("get_user", { id });
}

export async function updateUser(
  id: string,
  email?: string | null,
  name?: string | null,
  role?: string | null,
  language?: string | null,
): Promise<UserResponse> {
  return invokeCommand<UserResponse>("update_user", {
    id,
    email: email ?? null,
    name: name ?? null,
    role: role ?? null,
    language: language ?? null,
  });
}

export async function setUserActive(
  id: string,
  isActive: boolean,
): Promise<UserResponse> {
  return invokeCommand<UserResponse>("set_user_active", {
    id,
    is_active: isActive,
  });
}
