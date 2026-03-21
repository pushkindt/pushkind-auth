import "../styles/shell.css";
import { loadBootstrapPage } from "../lib/loadBootstrap";
import {
  AuthSignupPage,
  type SignupPageBootstrap,
} from "../pages/AuthSignupPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadBootstrapPage<SignupPageBootstrap>(
    rootElement,
    "/auth/bootstrap/signup",
    (bootstrap) => <AuthSignupPage {...bootstrap} />,
  );
}
