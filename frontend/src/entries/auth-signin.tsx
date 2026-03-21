import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "../styles/shell.css";
import { readBootstrap } from "../lib/readBootstrap";
import {
  AuthSigninPage,
  type SigninPageBootstrap,
} from "../pages/AuthSigninPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  const bootstrap = readBootstrap<SigninPageBootstrap>("frontend-bootstrap");

  createRoot(rootElement).render(
    <StrictMode>
      <AuthSigninPage {...bootstrap} />
    </StrictMode>,
  );
}
