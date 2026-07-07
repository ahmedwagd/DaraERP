import { invokeCommand } from "../lib/tauri";

export interface User {
  id: string;
  name: string | null;
  email: string;
  role: string;
  language: string;
}

export async function login(email: string, password: string): Promise<User> {
  return invokeCommand<User>("login", { email, password });
}

export async function refreshSession(): Promise<User> {
  return invokeCommand<User>("refresh_session");
}

export async function getCurrentUser(): Promise<User | null> {
  return invokeCommand<User | null>("get_current_user");
}

export async function logout(): Promise<void> {
  return invokeCommand<void>("logout");
}
