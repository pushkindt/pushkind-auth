import "../styles/shell.css";
import { loadBootstrapPage } from "../lib/loadBootstrap";
import { AuthSignupPage, type SignupPageData } from "../pages/AuthSignupPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadBootstrapPage<SignupPageData>(
    rootElement,
    "/api/v1/hubs",
    (hubs) => <AuthSignupPage hubs={hubs} />,
  );
}
