import { describe, expect, it } from "vitest";

import { toFieldErrorMap, type ApiMutationError } from "./api";

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
