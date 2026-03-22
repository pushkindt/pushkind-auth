# Plan: Generic Client Data APIs

## References
- Feature spec:
  [../spec/features/generic-page-data-apis.md](../spec/features/generic-page-data-apis.md)
- Architecture decision:
  [../spec/decisions/0003-generic-page-data-apis.md](../spec/decisions/0003-generic-page-data-apis.md)

## Objective
Replace route-specific bootstrap fetches with narrower client data APIs that
React pages consume after loading static Vite documents.

## Fixed Implementation Decisions
- The frontend source code WILL remain in `frontend/`.
- Static HTML documents WILL continue to be served by Actix after access
  checks.
- React WILL continue to fetch page data client-side.
- Redirects WILL remain backend-owned.
- The initial resource namespace WILL be `/api/v1/`.
- The initial frontend-owned GET surface WILL expose hubs, IAM, and hub-scoped
  menu items.
- GET endpoints WILL NOT be used to expose alerts.
- Frontend-owned POST flows SHOULD return structured JSON errors.
- Existing bootstrap endpoints WILL be migrated to the new endpoints before
  they are removed.
- Existing `GET /api/v1/id` and `GET /api/v1/users` MUST be assessed for
  reuse before adding overlapping endpoints.

## Target Endpoint Set

### Shared Resource Endpoints
- `GET /api/v1/hubs`
  Purpose:
  list hubs available to the current viewer or current flow.
  Used by:
  sign-in, sign-up, and any app UI that needs hub selection.
- `GET /api/v1/id`
  Purpose:
  existing current-user endpoint that MUST be assessed as the base identity
  source for the frontend.
  Initial payload:
  current user data as already exposed today.
  Reuse rule:
  prefer extending or adapting this endpoint over introducing
  `GET /api/v1/iam` if the needed IAM fields fit its responsibility.
- `GET /api/v1/iam`
  Purpose:
  only introduced if `GET /api/v1/id` cannot cleanly cover the required IAM
  contract.
  Initial payload:
  authenticated user info, roles, currently selected hub if applicable.
  Used by:
  app shell, auth gating, and role-aware React rendering.
- `GET /api/v1/hubs/{hub_id}/menu-items`
  Purpose:
  return menu/navigation items for one hub.
  Used by:
  authenticated app shell and dashboard navigation.

### Page-Specific Endpoints That May Remain Necessary
- `GET /api/v1/admin/dashboard`
  Purpose:
  return admin-only collections not naturally modeled by hubs, identity/IAM,
  menu items, or the existing user-list endpoint.
  Initial payload:
  roles, admin menu management data, and only the admin-specific data that is
  not already covered by `GET /api/v1/users`.
- `GET /api/v1/users`
  Reuse assessment:
  SHOULD be reused for the admin dashboard users table if its existing payload
  and filtering semantics are sufficient.
  Duplication rule:
  do not duplicate user-list functionality in `/api/v1/admin/dashboard`.

### Existing Endpoint Mapping
- `/auth/bootstrap/signin` -> replaced by:
  `GET /api/v1/hubs`
- `/auth/bootstrap/signup` -> replaced by:
  `GET /api/v1/hubs`
- `/bootstrap/basic` -> replaced by:
  `GET /api/v1/id` or `GET /api/v1/iam`,
  `GET /api/v1/hubs/{hub_id}/menu-items`
- `/bootstrap/admin` -> replaced by:
  `GET /api/v1/id` or `GET /api/v1/iam`,
  `GET /api/v1/hubs/{hub_id}/menu-items`,
  `GET /api/v1/users`,
  and `GET /api/v1/admin/dashboard` for the remaining admin-specific data

## DTO Split
- `HubListItemDto`
  Fields:
  `id`, `name`
- `IamDto`
  Fields:
  current user identity, roles, current hub summary, editable current-user
  profile fields needed by the basic dashboard
- `MenuItemDto`
  Fields:
  `name`, `url`
- `AdminDashboardDto`
  Fields:
  admin-only collections such as roles, hubs, and admin menu items.
  Reuse note:
  user lists SHOULD come from `GET /api/v1/users` if possible.

## Frontend Fetch Model
- Auth pages WILL fetch:
  `GET /api/v1/hubs`
- Non-admin dashboard WILL fetch:
  `GET /api/v1/id` or `GET /api/v1/iam`
  and `GET /api/v1/hubs/{hub_id}/menu-items`
- Admin dashboard WILL fetch:
  `GET /api/v1/id` or `GET /api/v1/iam`,
  `GET /api/v1/hubs/{hub_id}/menu-items`,
  `GET /api/v1/users`,
  and `GET /api/v1/admin/dashboard`
- The shared API client SHOULD support parallel fetches and merge responses
  into page props before rendering.
- POST interactions owned by React SHOULD prefer JSON request/response flows
  over redirect-plus-flash flows.

## Implementation Sequence

### Phase 1: Shared DTO Design
Deliverables:
- Introduce DTOs for:
  `HubListItemDto`,
  `IamDto`,
  `MenuItemDto`,
  `AdminDashboardDto`.
- Assess whether `api_v1_id` can be extended or adapted to satisfy the IAM
  contract without breaking existing API consumers.
- Assess whether `api_v1_users` can be reused directly for the admin dashboard
  users table.
- Define exactly which current bootstrap fields map to:
  hubs,
  IAM,
  hub-scoped menu items,
  admin dashboard data.
- Define how the frontend derives the selected hub id needed for
  `/api/hubs/{hub_id}/menu-items`.

Exit criteria:
- The data model clearly distinguishes reusable resource DTOs from
  page-specific resource DTOs.
- The plan explicitly states which functionality is reused from
  `api_v1_id` and `api_v1_users`, and which requires new endpoints.
- The target endpoints and DTO ownership are fixed in code comments and plan
  docs, not left as assumptions.

### Phase 2: Auth Resource Endpoints
Deliverables:
- Add `GET /api/v1/hubs` for auth usage.
- Update the auth frontend loader so:
  sign-in fetches `/api/v1/hubs`;
  sign-up fetches `/api/v1/hubs`.
- Keep behavior identical for:
  hub selection,
  and authenticated-user rejection.
- Keep `next` as request URL state carried through the frontend and validated
  only by the backend route that actually performs the redirect.
- Keep `/auth/bootstrap/signin` and `/auth/bootstrap/signup` temporarily as
  compatibility endpoints until the frontend is switched.

Exit criteria:
- Auth pages render entirely from the new `/api/v1/...` endpoints.
- The old auth bootstrap endpoints are unused by the frontend and can be marked
  deprecated.

### Phase 3: App Resource Endpoints
Deliverables:
- Reuse or extend `GET /api/v1/id` for identity/IAM data, or add
  `GET /api/v1/iam` only if that reuse is not clean.
- Add `GET /api/v1/hubs/{hub_id}/menu-items`.
- Reuse `GET /api/v1/users` for the admin users table if feasible.
- Add `GET /api/v1/admin/dashboard` only for admin-only collections that are
  not covered by reused endpoints.
- Update the dashboard frontend loaders so:
  basic dashboard fetches `/api/v1/id` or `/api/v1/iam`,
  and `/api/v1/hubs/{hub_id}/menu-items`;
  admin dashboard fetches `/api/v1/id` or `/api/v1/iam`,
  `/api/v1/hubs/{hub_id}/menu-items`,
  `/api/v1/users`,
  and `/api/v1/admin/dashboard`.
- Keep admin authorization boundaries explicit:
  non-admins MUST NOT receive `/api/v1/admin/dashboard`,
  admins MAY receive shared resources plus admin dashboard data.
- Keep `/bootstrap/basic` and `/bootstrap/admin` temporarily as compatibility
  endpoints until the frontend is switched.

Exit criteria:
- Dashboard pages load entirely through the new `/api/v1/...` endpoints.
- The old dashboard bootstrap endpoints are unused by the frontend and can be
  marked deprecated.

### Phase 4: JSON POST Responses
Deliverables:
- Choose the first React-owned POST flows to migrate from redirect responses to
  JSON responses.
- Add structured JSON success/error responses for those POST endpoints.
- Return field-level validation errors in a stable DTO shape.
- Update the corresponding frontend forms to render returned JSON errors
  without page reload.
- Document which remaining flows still depend on redirect-plus-flash behavior.

Exit criteria:
- At least one React-owned interaction no longer depends on flash middleware.
- The plan explicitly states which flows still require
  `actix_web_flash_messages`.

### Phase 4.1: Remaining JSON POST Migrations
Deliverables:
- Migrate the remaining React-owned POST endpoints from redirect-plus-flash
  responses to JSON responses.
- Cover at least these routes:
  `POST /auth/login`,
  `POST /auth/register`,
  `POST /admin/role/add`,
  `POST /admin/role/delete/{id}`,
  `POST /admin/hub/add`,
  `POST /admin/hub/delete/{id}`,
  `POST /admin/menu/add`,
  `POST /admin/menu/delete/{id}`,
  `POST /admin/user/update/{id}`,
  `POST /admin/user/delete/{id}`.
- Reuse the Phase 4 mutation DTOs for:
  success messages,
  top-level error messages,
  and field-level validation errors.
- Update the corresponding React forms so they submit with `fetch`,
  render returned field errors inline,
  and refresh or mutate local page state without full-page reloads.
- Define the state-refresh rule for each flow:
  whether the page refetches the affected resource endpoint,
  or updates the in-memory React state directly after a successful mutation.
- Keep authorization failures explicit in JSON responses:
  unauthenticated requests MUST still fail through the auth middleware,
  unauthorized admin mutations MUST return the appropriate HTTP error.
- Document which flows, if any, still require `actix_web_flash_messages`
  after this phase.

Exit criteria:
- All React-owned POST interactions use JSON request/response handling.
- Login and registration no longer depend on redirect-plus-flash error flows.
- Admin CRUD forms no longer require full-page reloads to surface validation
  or success feedback.
- `actix_web_flash_messages` remains only for non-React or legacy flows, or is
  explicitly identified as removable.

### Phase 5: Cleanup
Deliverables:
- Remove:
  `/auth/bootstrap/signin`,
  `/auth/bootstrap/signup`,
  `/bootstrap/basic`,
  `/bootstrap/admin`.
- Remove temporary compatibility DTOs and loaders.
- Collapse transitional bootstrap types into the final resource DTOs.
- Remove flash-message usage only from flows that have fully migrated to JSON
  POST handling.
- Update docs to reflect the new page-data API structure.

Exit criteria:
- The frontend uses one consistent page-data loading pattern.
- No React page depends on a route-specific `/bootstrap/...` endpoint.

## Verification
- `cargo build --all-features --verbose`
- `cargo test --all-features --verbose`
- `cargo clippy --all-features --tests -- -Dwarnings`
- `cargo fmt --all -- --check`
- `cd frontend && npm run typecheck`
- `cd frontend && npm run build`
