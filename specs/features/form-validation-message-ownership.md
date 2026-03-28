# Form Validation Message Ownership

## Status
Stable

## Date
2026-03-27

## Summary
Move field-level validation messages out of `src/routes/mod.rs` and into the
form layer so each form owns the copy returned for its validation failures.

## Problem
`src/routes/mod.rs` currently maps validator error codes and field names to
user-facing messages. That makes route helpers responsible for form-specific
copy and creates a second place that must change whenever a form adds or edits
validation behavior.

## Goals
- Make `src/forms` the owner of field-level validation messages.
- Make `FormError` display strings localized in Russian.
- Keep the current user-facing Russian validation copy unchanged.
- Preserve the existing JSON error shape returned by mutation routes.

## Non-Goals
- Changing validation rules or domain constraints.
- Changing HTTP status codes or top-level error messages.
- Introducing API-stable guarantees for validation copy.

## Functional Requirements

### 1. Validator-Backed Messages
- Form structs under `src/forms` MUST define explicit validator messages for
  fields whose responses currently rely on `validation_message`.
- Route helpers MUST read validator-provided messages instead of matching on
  field names and validator codes.

### 2. Form Conversion Errors
- Messages for `FormError` variants produced after validator execution MUST be
  defined in the forms layer rather than in `src/routes/mod.rs`.
- `FormError::to_string()` output MUST use localized Russian copy for both
  validator-backed and conversion-backed failures.

### 3. Route Simplicity
- `src/routes/mod.rs` MUST remain responsible only for translating form-layer
  errors into API DTOs.
- Route code MUST NOT encode per-field validation copy.

## Acceptance Criteria
- `validation_message` is removed from `src/routes/mod.rs`.
- The same field-level messages continue to be returned for invalid
  `email`, `password`, `hub_id`, `name`, `url`, and `roles` inputs.
- `FormError` display output is localized for both `Validation` and
  single-field conversion errors.
- New unit tests cover message extraction from validator errors and
  `FormError` conversion errors.
