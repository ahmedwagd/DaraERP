/* eslint-disable react-refresh/only-export-components */
import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
import { useTranslation } from "react-i18next";
import * as authService from "../services/auth";
import type { User } from "../services/auth";

interface AuthContextValue {
  user: User | null;
  loading: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const { i18n } = useTranslation();

  useEffect(() => {
    const restore = async () => {
      try {
        const current = await authService.getCurrentUser();
        if (current) {
          setUser(current);
          if (current.language && current.language !== i18n.language) {
            i18n.changeLanguage(current.language);
          }
          return;
        }
        const refreshed = await authService.refreshSession();
        setUser(refreshed);
        if (refreshed.language && refreshed.language !== i18n.language) {
          i18n.changeLanguage(refreshed.language);
        }
      } catch {
        setUser(null);
      } finally {
        setLoading(false);
      }
    };
    restore();
  }, [i18n]);

  const login = useCallback(
    async (email: string, password: string) => {
      const u = await authService.login(email, password);
      setUser(u);
      if (u.language && u.language !== i18n.language) {
        i18n.changeLanguage(u.language);
      }
    },
    [i18n],
  );

  const logout = useCallback(async () => {
    await authService.logout();
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider value={{ user, loading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return ctx;
}
