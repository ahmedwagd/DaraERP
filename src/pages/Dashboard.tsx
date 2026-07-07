import { useTranslation } from "react-i18next";
import { useAuth } from "../contexts/AuthContext";

function Dashboard() {
  const { t } = useTranslation();
  const { user } = useAuth();

  if (!user) {
    return null;
  }

  return (
    <div>
      <h1>{t("auth.dashboardTitle")}</h1>
      <p>
        {t("auth.welcome")}, {user.name ?? user.email}
      </p>
      <p>
        {t("role." + user.role)}
      </p>
    </div>
  );
}

export default Dashboard;
