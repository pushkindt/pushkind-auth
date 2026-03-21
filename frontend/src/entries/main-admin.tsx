import "../styles/shell.css";
import { loadBootstrapPage } from "../lib/loadBootstrap";
import {
  MainAdminPage,
  type AdminDashboardBootstrap,
} from "../pages/MainAdminPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadBootstrapPage<AdminDashboardBootstrap>(
    rootElement,
    "/bootstrap/admin",
    (bootstrap) => <MainAdminPage {...bootstrap} />,
  );
}
