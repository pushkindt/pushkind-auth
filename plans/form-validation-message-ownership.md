# Plan: Form Validation Message Ownership

## References
- Feature spec:
  [../specs/features/form-validation-message-ownership.md](../specs/features/form-validation-message-ownership.md)

## Objective
Centralize form validation copy in `src/forms` and keep route helpers limited
to shaping those errors for HTTP responses.

## Fixed Implementation Decisions
- Existing user-facing validation text WILL stay unchanged.
- Validator-backed messages WILL be declared directly on form fields.
- `FormError` WILL expose field-level message data for route helpers.
- `FormError` display output WILL be localized in Russian.
- No ADR is required because this change does not alter the project
  architecture.

## Implementation Sequence

### Phase 1: Form Message Ownership
Deliverables:
- Add explicit validator messages to forms that currently rely on route-level
  message matching.
- Add form-layer mapping for `FormError` variants that are not produced by the
  validator derive macros.
- Localize `FormError` display strings so log and service-conversion messages
  match the rest of the form layer.

Exit criteria:
- `src/forms` can fully describe field-level validation errors without route
  logic.

### Phase 2: Route Simplification
Deliverables:
- Remove `validation_message` and related route-level mapping code.
- Update the shared route helper to translate form-owned messages into API DTOs.

Exit criteria:
- `src/routes/mod.rs` no longer contains per-field validation copy.

### Phase 3: Verification
Deliverables:
- Add unit tests covering validator message extraction and conversion error
  mapping.
- Add unit tests covering localized `FormError` display output.
- Run targeted formatting and tests for the touched modules.

Exit criteria:
- Tests confirm the existing field-level messages still come back from the form
  layer.
