import "../styles/shell.css";
import { loadComposedPage } from "../lib/loadBootstrap";
import { MainAdminPage } from "../pages/MainAdminPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadComposedPage(
    rootElement,
    async () => undefined,
    () => <MainAdminPage />,
  );
}
