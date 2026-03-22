import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./redirect", () => ({
  redirectTo: vi.fn(),
}));

import {
  RedirectResponseError,
  postForm,
  postJson,
  toFieldErrorMap,
  type ApiMutationError,
} from "./api";
import { redirectTo } from "./redirect";

function makeResponse({
  contentType = "application/json",
  redirected = false,
  url = "http://localhost/api/test",
  status = 200,
  payload = {},
}: {
  contentType?: string;
  redirected?: boolean;
  url?: string;
  status?: number;
  payload?: unknown;
}): Response {
  return {
    headers: new Headers({ "content-type": contentType }),
    redirected,
    url,
    status,
    ok: status >= 200 && status < 300,
    json: vi.fn().mockResolvedValue(payload),
  } as unknown as Response;
}

beforeEach(() => {
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("toFieldErrorMap", () => {
  it("maps field errors by field name", () => {
    const error: ApiMutationError = {
      message: "Validation failed.",
      field_errors: [
        { field: "email", message: "Invalid email." },
        { field: "password", message: "Required." },
      ],
    };

    expect(toFieldErrorMap(error)).toEqual({
      email: "Invalid email.",
      password: "Required.",
    });
  });

  it("keeps the last message for duplicate fields", () => {
    const error: ApiMutationError = {
      message: "Validation failed.",
      field_errors: [
        { field: "email", message: "Invalid email." },
        { field: "email", message: "Email already exists." },
      ],
    };

    expect(toFieldErrorMap(error)).toEqual({
      email: "Email already exists.",
    });
  });
});

describe("postForm", () => {
  it("returns the JSON payload for successful mutation responses", async () => {
    vi.mocked(fetch).mockResolvedValue(
      makeResponse({
        payload: {
          message: "Saved.",
          redirect_to: null,
        },
      }),
    );

    await expect(
      postForm("/user/save", new URLSearchParams()),
    ).resolves.toEqual({
      message: "Saved.",
      redirect_to: null,
    });
  });

  it("redirects to sign-in when middleware returns HTML instead of JSON", async () => {
    const response = makeResponse({
      contentType: "text/html; charset=utf-8",
      redirected: true,
      url: "http://localhost/auth/signin?next=%2F",
    });

    vi.mocked(fetch).mockResolvedValue(response);

    await expect(
      postForm("/user/save", new URLSearchParams()),
    ).rejects.toBeInstanceOf(RedirectResponseError);
    expect(vi.mocked(redirectTo)).toHaveBeenCalledWith(
      "http://localhost/auth/signin?next=%2F",
    );
    expect(response.json).not.toHaveBeenCalled();
  });
});

describe("postJson", () => {
  it("redirects instead of decoding HTML when admin bootstrap requests are redirected", async () => {
    const response = makeResponse({
      contentType: "text/html; charset=utf-8",
      redirected: true,
      url: "http://localhost/auth/signin?next=%2Fadmin%2Fuser%2Fmodal%2F1",
    });

    vi.mocked(fetch).mockResolvedValue(response);

    await expect(postJson("/admin/user/modal/1")).rejects.toBeInstanceOf(
      RedirectResponseError,
    );
    expect(vi.mocked(redirectTo)).toHaveBeenCalledWith(
      "http://localhost/auth/signin?next=%2Fadmin%2Fuser%2Fmodal%2F1",
    );
    expect(response.json).not.toHaveBeenCalled();
  });
});
