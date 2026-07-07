import { useState } from "react";
import { useTranslation } from "react-i18next";

interface LoginProps {
  onLogin: (email: string, password: string) => Promise<void>;
}

function Login({ onLogin }: LoginProps) {
  const { t, i18n } = useTranslation();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!email.trim() || !password) {
      setError(t("auth.invalidCredentials"));
      return;
    }

    setLoading(true);
    try {
      await onLogin(email.trim(), password);
    } catch (err: unknown) {
      const msg =
        err && typeof err === "object" && "code" in err
          ? String((err as { code: string }).code)
          : "";
      if (msg === "AUTH_INVALID_CREDENTIALS") {
        setError(t("auth.invalidCredentials"));
      } else {
        setError(t("common.error"));
      }
    } finally {
      setLoading(false);
    }
  };

  const toggleLanguage = () => {
    const next = i18n.language === "en" ? "ar" : "en";
    i18n.changeLanguage(next);
  };

  return (
    <div dir={i18n.language === "ar" ? "rtl" : "ltr"}>
      <h1>{t("auth.login")}</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="email">{t("auth.email")}</label>
          <input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            disabled={loading}
            autoComplete="email"
          />
        </div>
        <div>
          <label htmlFor="password">{t("auth.password")}</label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={loading}
            autoComplete="current-password"
          />
        </div>
        {error && <p style={{ color: "red" }}>{error}</p>}
        <button type="submit" disabled={loading}>
          {loading ? t("auth.loggingIn") : t("auth.loginButton")}
        </button>
      </form>
      <button onClick={toggleLanguage}>
        {i18n.language === "en" ? t("auth.arabic") : t("auth.english")}
      </button>
    </div>
  );
}

export default Login;
