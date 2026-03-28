# Plan: Auth Sign-In User Stories

## References
- Feature spec:
  [../specs/features/e2e-user-stories.md](../specs/features/e2e-user-stories.md)

## Objective
Replace the current sign-in-focused integration coverage with end-to-end tests
for the main production user stories for both admin and non-admin users.

## Fixed Implementation Decisions
- The existing Actix integration harness in `tests/common/mod.rs` WILL remain
  the entry point for HTTP-driven tests.
- Test data WILL be seeded through repository helpers instead of raw SQL.
- The tests WILL assert the current JSON login API and current dashboard shell
  HTML assets.
- No ADR is required because this change only expands test coverage.

## Implementation Sequence

### Phase 1: Fixture Setup
Deliverables:
- Extend test helpers to seed the default hub with both an admin user and a
  regular user.
- Expose deterministic credentials and the default hub id for reuse in
  integration tests.

Exit criteria:
- `tests/auth_signin.rs` can log in as either seeded user without duplicating
  fixture logic.

### Phase 2: Integration Coverage
Deliverables:
- Keep a logged-out `GET /auth/signin` test.
- Add an admin user-story test that covers login, authenticated sign-in
  redirect, dashboard shell selection, admin API access, role creation and
  deletion, hub creation and deletion, menu creation and deletion, user modal
  bootstrap, and user updates.
- Add a non-admin user-story test that covers login, authenticated sign-in
  redirect, dashboard shell selection, forbidden admin API access, and
  self-service profile updates.

Exit criteria:
- The sign-in test file covers both user roles end to end.

### Phase 3: Verification
Deliverables:
- Run `cargo fmt`.
- Run `cargo test --test e2e`.

Exit criteria:
- Formatting and the targeted integration test pass locally.
