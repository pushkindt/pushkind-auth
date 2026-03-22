import "../styles/shell.css";
import { loadBootstrapPage } from "../lib/loadBootstrap";
import { AuthSigninPage, type SigninPageData } from "../pages/AuthSigninPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadBootstrapPage<SigninPageData>(
    rootElement,
    "/api/v1/hubs",
    (hubs) => <AuthSigninPage hubs={hubs} />,
  );
}
