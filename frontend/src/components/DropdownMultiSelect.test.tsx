import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { DropdownMultiSelect } from "./DropdownMultiSelect";

describe("DropdownMultiSelect", () => {
  it("renders the provided control id on the toggle button", () => {
    const markup = renderToStaticMarkup(
      <DropdownMultiSelect
        id="role-selector"
        options={[{ value: "1", label: "Admin" }]}
        selectedValues={[]}
        onChange={() => {}}
      />,
    );

    expect(markup).toContain('id="role-selector"');
    expect(markup).toContain('aria-controls="role-selector-menu"');
  });

  it("does not nest the clear button inside the toggle button", () => {
    const markup = renderToStaticMarkup(
      <DropdownMultiSelect
        options={[{ value: "1", label: "Admin" }]}
        selectedValues={["1"]}
        onChange={() => {}}
        clearable
      />,
    );

    expect(markup).toContain('class="auth-dropdown-multiselect-toggle"');
    expect(markup).toContain('class="auth-dropdown-multiselect-clear"');
    expect(markup).toContain("</button><span");
  });
});
