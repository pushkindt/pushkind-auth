import "../styles/shell.css";
import { loadBootstrapPage } from "../lib/loadBootstrap";
import {
  MainBasicPage,
  type BasicDashboardBootstrap,
} from "../pages/MainBasicPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadBootstrapPage<BasicDashboardBootstrap>(
    rootElement,
    "/bootstrap/basic",
    (bootstrap) => <MainBasicPage {...bootstrap} />,
  );
}
