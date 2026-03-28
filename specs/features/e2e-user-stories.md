# Auth Sign-In User Stories

## Status
Stable

## Date
2026-03-28

## Summary
Extend the end-to-end coverage for the production authentication and
authorization stories so the test suite covers both an admin user and a
regular non-admin user end to end, including the key admin mutation flows and
the regular user's self-service profile update flow.

## Problem
`tests/e2e.rs` currently covers only the sign-in page and basic
post-login role-based landing behavior. It does not verify the primary
admin-management mutations or the non-admin self-service profile-editing flow
through the production HTTP routes.

## Goals
- Cover the public sign-in page for logged-out visitors.
- Cover successful login for an admin user and a non-admin user.
- Verify that authenticated users are redirected away from `/auth/signin`.
- Verify that admin and non-admin users land on different dashboard shells.
- Verify admin-only API access is allowed for admins and forbidden for
  non-admin users.
- Verify an admin can create and delete roles, hubs, and menu items.
- Verify an admin can load the user-edit bootstrap payload and update another
  user's profile and role assignments.
- Verify a non-admin can update their own profile information.

## Non-Goals
- Changing authentication, authorization, or session behavior.
- Adding new route handlers or new UI behavior.
- Reworking the shared integration-test harness beyond data setup needed for
  these scenarios.

## Functional Requirements

### 1. Logged-Out Sign-In
- `GET /auth/signin` MUST return `200 OK` and HTML content for an anonymous
  visitor.

### 2. Admin Story
- A seeded admin user MUST be able to log in successfully through
  `POST /auth/login`.
- After login, `GET /auth/signin` MUST redirect the authenticated admin to `/`.
- `GET /` MUST return the admin dashboard shell.
- `GET /api/v1/admin/dashboard` MUST return `200 OK` for the admin session.
- The admin MUST be able to create a role through `POST /admin/role/add` and
  delete it through `POST /admin/role/delete/{role_id}`.
- The admin MUST be able to create a hub through `POST /admin/hub/add` and
  delete it through `POST /admin/hub/delete/{hub_id}`.
- The admin MUST be able to create a menu item through `POST /admin/menu/add`
  and delete it through `POST /admin/menu/delete/{menu_id}`.
- The admin MUST be able to fetch user modal bootstrap data through
  `POST /admin/user/modal/{user_id}`.
- The admin MUST be able to update another user through
  `POST /admin/user/update/{user_id}`.

### 3. Non-Admin Story
- A seeded regular user MUST be able to log in successfully through
  `POST /auth/login`.
- After login, `GET /auth/signin` MUST redirect the authenticated user to `/`.
- `GET /` MUST return the basic dashboard shell.
- `GET /api/v1/admin/dashboard` MUST return `403 Forbidden` for the regular
  user session.
- The regular user MUST be able to update their own profile through
  `POST /user/save`.

## Acceptance Criteria
- `tests/e2e.rs` contains active integration tests for the anonymous,
  admin, and non-admin scenarios, including the production mutation flows
  described above.
- The tests use deterministic fixture data created through the repository test
  helpers.
- Targeted integration execution passes with `cargo test --test e2e`.
