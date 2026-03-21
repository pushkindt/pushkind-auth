import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "../styles/shell.css";
import { readBootstrap } from "../lib/readBootstrap";
import {
  MainAdminPage,
  type AdminDashboardBootstrap,
} from "../pages/MainAdminPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  const bootstrap =
    readBootstrap<AdminDashboardBootstrap>("frontend-bootstrap");

  createRoot(rootElement).render(
    <StrictMode>
      <MainAdminPage {...bootstrap} />
    </StrictMode>,
  );
}
