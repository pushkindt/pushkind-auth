import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "../styles/shell.css";
import { readBootstrap } from "../lib/readBootstrap";
import {
  MainBasicPage,
  type BasicDashboardBootstrap,
} from "../pages/MainBasicPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  const bootstrap =
    readBootstrap<BasicDashboardBootstrap>("frontend-bootstrap");

  createRoot(rootElement).render(
    <StrictMode>
      <MainBasicPage {...bootstrap} />
    </StrictMode>,
  );
}
