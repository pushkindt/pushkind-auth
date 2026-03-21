import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "../styles/shell.css";
import { readBootstrap } from "../lib/readBootstrap";
import {
  AuthSignupPage,
  type SignupPageBootstrap,
} from "../pages/AuthSignupPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  const bootstrap = readBootstrap<SignupPageBootstrap>("frontend-bootstrap");

  createRoot(rootElement).render(
    <StrictMode>
      <AuthSignupPage {...bootstrap} />
    </StrictMode>,
  );
}
