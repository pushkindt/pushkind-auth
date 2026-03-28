import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { UserMenuDropdown } from "./UserMenuDropdown";

describe("UserMenuDropdown", () => {
  it("renders local items before fetched items and keeps logout last", () => {
    const html = renderToStaticMarkup(
      createElement(UserMenuDropdown, {
        currentUserEmail: "user@example.com",
        localItems: [
          { name: "Главная", url: "/", iconClass: "bi bi-house" },
          { name: "Профиль", url: "/profile", iconClass: "bi bi-person" },
        ],
        fetchedItems: [
          { name: "Заказы", url: "https://example.com/orders" },
          { name: "Отчеты", url: "https://example.com/reports" },
        ],
        logoutAction: "/auth/logout",
      }),
    );

    expect(html.indexOf("Главная")).toBeLessThan(html.indexOf("Профиль"));
    expect(html.indexOf("Профиль")).toBeLessThan(html.indexOf("Заказы"));
    expect(html.indexOf("Заказы")).toBeLessThan(html.indexOf("Отчеты"));
    expect(html.indexOf("Отчеты")).toBeLessThan(html.indexOf("Выйти"));
  });
});
