import "../styles/shell.css";
import { loadComposedPage } from "../lib/loadBootstrap";
import { MainBasicPage } from "../pages/MainBasicPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadComposedPage(
    rootElement,
    async () => undefined,
    () => <MainBasicPage />,
  );
}
