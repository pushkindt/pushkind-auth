import "../styles/shell.css";
import type { ApiIam, ApiMenuItem } from "../lib/api";
import { fetchJson } from "../lib/api";
import { loadComposedPage } from "../lib/loadBootstrap";
import {
  MainBasicPage,
  type BasicDashboardPageData,
} from "../pages/MainBasicPage";

const rootElement = document.getElementById("react-root");

if (rootElement) {
  void loadComposedPage<BasicDashboardPageData>(
    rootElement,
    async () => {
      const iam = await fetchJson<ApiIam>("/api/v1/iam");
      const menu = await fetchJson<ApiMenuItem[]>(
        `/api/v1/hubs/${iam.current_hub.id}/menu-items`,
      );

      return { iam, menu };
    },
    (pageData) => <MainBasicPage {...pageData} />,
  );
}
