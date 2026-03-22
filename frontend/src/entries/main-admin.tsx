import "../styles/shell.css";
import type {
  ApiAdminDashboard,
  ApiIam,
  ApiMenuItem,
  ApiUserListItem,
  DashboardUser,
} from "../lib/api";
import { fetchJson } from "../lib/api";
import { loadComposedPage } from "../lib/loadBootstrap";
import {
  MainAdminPage,
  type AdminDashboardPageData,
} from "../pages/MainAdminPage";

function mapUsers(users: ApiUserListItem[]): DashboardUser[] {
  return users.map((user) => ({
    id: Number(user.sub),
    email: user.email,
    name: user.name,
    roles: user.roles,
  }));
}

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadComposedPage<AdminDashboardPageData>(
    rootElement,
    async () => {
      const iam = await fetchJson<ApiIam>("/api/v1/iam");
      const [menu, users, admin] = await Promise.all([
        fetchJson<ApiMenuItem[]>(
          `/api/v1/hubs/${iam.current_hub.id}/menu-items`,
        ),
        fetchJson<ApiUserListItem[]>("/api/v1/users"),
        fetchJson<ApiAdminDashboard>("/api/v1/admin/dashboard"),
      ]);

      return { iam, menu, users: mapUsers(users), admin };
    },
    (pageData) => <MainAdminPage {...pageData} />,
  );
}
