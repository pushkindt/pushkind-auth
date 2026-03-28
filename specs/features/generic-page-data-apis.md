# Generic Client Data APIs

## Status
Stable

## Date
2026-03-27

## Summary
Replace route-specific bootstrap endpoints with narrower client data APIs that
React pages fetch client-side. The backend MUST continue to own auth,
authorization, validation, redirects, and persistence, but page initialization
data SHOULD come from specific resource endpoints rather than page-shaped
bootstrap payloads.

## Problem
The current static-Vite frontend still relies on dedicated bootstrap endpoints
such as `/auth/bootstrap/signin`, `/auth/bootstrap/signup`, `/bootstrap/basic`,
and `/bootstrap/admin`. This works, but it keeps the frontend/backend contract
coupled to specific page documents and duplicates patterns for:

- redirect continuation state such as `next`
- current user and hub context
- page-specific initialization payloads

If that pattern continues, each new page will need another custom bootstrap
endpoint even when the actual data being requested is simpler and more
resource-like.

## Goals
- Define a clearer client data contract for React-owned pages.
- Reduce the need for page-specific bootstrap endpoints.
- Make client-side data loading more explicit and reusable across pages.
- Make POST error handling explicit through JSON responses instead of
  flash-message-driven page reload flows where practical.

## Non-Goals
- Introducing SPA routing.
- Moving business logic, authorization, or validation into React.
- Replacing server-side redirects with client-side navigation.
- Redesigning every existing form submission in one step.

## Desired End State
- React pages fetch initialization data from specific client data APIs.
- Shared concerns such as current identity, available hubs, and menu items come
  from reusable resource endpoints instead of ad hoc bootstrap payloads.
- Route handlers continue to select and serve static HTML documents after access
  checks.
- Validation and operation errors are returned from POST endpoints as JSON where
  the frontend owns the interaction flow.

## Functional Requirements

### 1. API Shape
- The backend SHOULD expose client data APIs under a predictable namespace such
  as `/api/v1/...`.
- New endpoints introduced for this migration MUST be versioned under
  `/api/v1/`.
- Endpoints SHOULD be resource-specific rather than page-specific.
- The initial endpoint set SHOULD include:
  a hub-list endpoint,
  an IAM endpoint,
  and a menu-items-for-hub endpoint.
- The frontend MUST be able to compose page rendering from these narrower
  endpoints without relying on HTML-embedded JSON.

### 2. Resource Boundaries
- `GET /api/v1/hubs` SHOULD return the hub collection needed by auth and app
  pages.
- Existing `GET /api/v1/id` SHOULD be assessed before introducing a separate
  IAM endpoint.
- If `GET /api/v1/id` can be extended without breaking its current contract, it
  SHOULD be reused as the current-user identity source instead of duplicating
  that responsibility in a new endpoint.
- If `GET /api/v1/id` cannot cleanly carry the required IAM data, a new
  `GET /api/v1/iam` endpoint MAY be introduced with an explicit reason.
- `GET /api/v1/hubs/{hub_id}/menu-items` or an equivalent hub-scoped endpoint
  SHOULD return navigation/menu items for the selected hub.
- Existing `GET /api/v1/users` SHOULD be assessed before introducing a separate
  admin-only user-list endpoint.
- Additional endpoints SHOULD be introduced only when the data cannot be
  expressed through the shared resource model.

### 3. Error Handling
- GET APIs MUST NOT be used to expose transient alerts.
- Where the frontend owns an interaction, POST endpoints SHOULD return JSON
  success or error payloads instead of relying on flash-message-based reloads.
- Validation errors MUST be structured and field-addressable for client-side
  rendering.
- Redirect-based flows MAY remain temporarily where the frontend has not yet
  been migrated to JSON POST handling.

### 4. Redirect Semantics
- Existing backend redirects MUST remain backend-controlled.
- Redirect targets such as `next` MUST continue to be validated on the server.
- Any client data contract used with redirect continuation MUST remain safe
  against open redirect behavior.

### 5. Migration Path
- Migration MAY happen incrementally.
- Existing route-specific bootstrap endpoints MAY coexist temporarily with the
  new resource endpoints.
- The target state SHOULD remove page-specific bootstrap transport when the
  resource endpoints reach parity.

## Backend Requirements
- Add or reuse versioned resource endpoints for hubs, identity/IAM, and
  hub-scoped menu items.
- Preserve existing GET route ownership for static HTML pages.
- Preserve existing form POST flows unless a route is explicitly migrated to
  JSON request/response handling.
- Preserve authorization boundaries for admin-only data.

## Frontend Requirements
- Replace ad hoc bootstrap fetches with a shared API client.
- Render explicit loading and fatal error states while page data is in flight.
- Keep page rendering route-based and non-SPA.

## On `actix_web_flash_messages`
- This direction makes `actix_web_flash_messages` less necessary, but only for
  flows migrated away from redirect-plus-flash handling.
- If POST endpoints return structured JSON errors and successes, those specific
  flows no longer need flash-message transport.
- Redirect-driven flows will still need either flash messages or a separate
  replacement until they are also migrated.

## Acceptance Criteria
- New React pages can initialize from specific client data APIs without
  requiring a one-off bootstrap endpoint.
- The frontend can fetch hubs, IAM, and menu items independently.
- At least one migrated POST interaction returns structured JSON errors instead
  of relying on flash-message-based page feedback.
- The design clearly states which flows still rely on redirects and flash
  middleware.

## Risks
- Over-generalizing shared APIs can still lead to vague contracts if endpoints
  stop matching real resources.
- Removing flash middleware too early can break remaining redirect-driven flows.
- Resource endpoints still need strong auth and route ownership, otherwise
  pages may fetch data they should not see.
