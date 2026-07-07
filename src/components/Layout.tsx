import { useTranslation } from "react-i18next";
import { Link, Outlet, useNavigate } from "react-router";
import { useAuth } from "../contexts/AuthContext";

function Layout() {
  const { t, i18n } = useTranslation();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate("/login");
  };

  const toggleLanguage = () => {
    const next = i18n.language === "en" ? "ar" : "en";
    i18n.changeLanguage(next);
  };

  if (!user) {
    return null;
  }

  return (
    <div dir={i18n.language === "ar" ? "rtl" : "ltr"}>
      <aside>
        <nav>
          <ul>
            <li>
              <Link to="/dashboard">{t("nav.dashboard")}</Link>
            </li>
            <li>
              <span style={{ opacity: 0.4 }}>{t("nav.customers")}</span>
            </li>
            <li>
              <span style={{ opacity: 0.4 }}>{t("nav.products")}</span>
            </li>
            <li>
              <span style={{ opacity: 0.4 }}>{t("nav.invoices")}</span>
            </li>
            <li>
              <span style={{ opacity: 0.4 }}>{t("nav.settings")}</span>
            </li>
          </ul>
        </nav>
      </aside>
      <header>
        <span>
          {user.name ?? user.email} ({t("role." + user.role)})
        </span>
        <button onClick={toggleLanguage}>
          {i18n.language === "en" ? t("auth.arabic") : t("auth.english")}
        </button>
        <button onClick={handleLogout}>{t("auth.logout")}</button>
      </header>
      <main>
        <Outlet />
      </main>
    </div>
  );
}

export default Layout;
