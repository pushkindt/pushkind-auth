import { afterEach, describe, expect, it } from "vitest";

import { getNextFromLocation, withNext } from "./auth";

describe("withNext", () => {
  it("returns the base path when next is missing", () => {
    expect(withNext("/auth/signin", null)).toBe("/auth/signin");
  });

  it("appends an encoded next parameter", () => {
    expect(withNext("/auth/signin", "/dashboard?tab=users")).toBe(
      "/auth/signin?next=%2Fdashboard%3Ftab%3Dusers",
    );
  });
});

describe("getNextFromLocation", () => {
  afterEach(() => {
    window.history.replaceState({}, "", "http://localhost/");
  });

  it("returns null when next is absent", () => {
    window.history.replaceState({}, "", "http://localhost/auth/signin");

    expect(getNextFromLocation()).toBeNull();
  });

  it("returns next when it is present and non-empty", () => {
    window.history.replaceState(
      {},
      "",
      "http://localhost/auth/signin?next=%2Fdashboard",
    );

    expect(getNextFromLocation()).toBe("/dashboard");
  });

  it("returns null when next is empty", () => {
    window.history.replaceState({}, "", "http://localhost/auth/signin?next=");

    expect(getNextFromLocation()).toBeNull();
  });
});
