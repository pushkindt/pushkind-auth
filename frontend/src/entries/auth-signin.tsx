import "../styles/shell.css";
import { loadBootstrapPage } from "../lib/loadBootstrap";
import {
  AuthSigninPage,
  type SigninPageBootstrap,
} from "../pages/AuthSigninPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadBootstrapPage<SigninPageBootstrap>(
    rootElement,
    "/auth/bootstrap/signin",
    (bootstrap) => <AuthSigninPage {...bootstrap} />,
  );
}
