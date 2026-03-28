# React Frontend Migration Preserving Existing UI

## Status
Stable

## Date
2026-03-27

## Summary
Migrate the current Tera-based frontend to React-managed UI components while
preserving the existing Bootstrap styling, route structure, copy, and user
flows. This change MUST modernize the frontend implementation without turning
`pushkind-auth` into a single-page application and without redesigning the UI.

## Problem
The current frontend is spread across Tera templates, inline JavaScript, and
HTMX attributes. That keeps the initial implementation small, but it makes the
UI harder to compose, test, and evolve as interactions grow more complex,
especially on the admin dashboard.

## Goals
- Introduce React as the frontend component model for user-facing pages.
- Preserve the current visual design, Bootstrap classes, HTML semantics, and
  user-visible copy.
- Preserve current server routes, session handling, redirect behavior, and
  authorization rules.
- Replace inline JavaScript and HTMX-driven interactions with React where those
  pages or widgets are migrated.
- Keep business logic in Rust service modules; React MUST remain a view layer.
- Move page initialization away from page-specific bootstrap transport toward
  narrower resource-style client data APIs under `/api/v1/`.

## Non-Goals
- Introducing client-side routing or SPA navigation.
- Redesigning pages, changing Bootstrap versions, or replacing Bootstrap with a
  new styling system.
- Moving validation, authorization, or persistence rules out of the backend.
- Changing route URLs, flash message semantics, or authentication/session
  mechanics beyond what is required to support React rendering.

## Current Baseline
The existing frontend surface is implemented in Tera templates:
- `templates/base.html`
- `templates/navigation.html`
- `templates/auth/login.html`
- `templates/auth/register.html`
- `templates/main/index.html`
- `templates/main/_basic.html`
- `templates/main/_admin.html`
- `templates/main/modal_body.html`

Current client behavior is a mix of:
- Bootstrap JS for dropdowns, modals, tooltips, and popovers.
- Inline JavaScript for password visibility, password confirmation, flash
  modals, and client-side filtering.
- HTMX for password recovery and admin user modal loading.

## In Scope
- Authentication pages: sign-in and sign-up.
- The authenticated index page for both basic and admin users.
- Shared navigation and flash message presentation.
- Admin dashboard interactions: filtering, user modal loading/editing, and
  create/delete actions for roles, hubs, and menu items.
- Frontend asset build and delivery needed to run React in production and local
  development.

## Out of Scope
- Changes to Diesel models, repository rules, service-layer business logic, or
  database schema.
- Changes to `/api/v1/*` semantics except where additional typed endpoints or
  view-model payloads are needed for the React UI.
- Replacing the auth/session model with token storage in the browser.

## Functional Requirements

### 1. Rendering Model
- The application MUST keep the existing server-owned route model.
- The application MUST NOT introduce client-side routing for `/`, `/auth/*`, or
  `/admin/*`.
- React MUST be introduced as page-level or island-level components mounted on
  the existing URLs.
- During rollout, pages MUST remain compatible with server rendering so the
  backend can continue to produce the initial HTML response.
- The long-term target MUST be full React rendering for each page without Tera
  templates and without SPA routing.

### 2. Markup And Style Preservation
- Migrated React components MUST preserve the current Bootstrap-based layout and
  visual hierarchy.
- Existing CSS class names, Bootstrap utility classes, form structure, modal
  structure, and navigation structure MUST remain stable unless a deviation is
  explicitly documented.
- User-visible Russian copy SHOULD remain unchanged except for bug fixes or
  accessibility corrections.
- Existing favicon, Bootstrap Icons usage, and current custom styling in the
  base layout MUST continue to work.

### 3. Behavioral Parity
- `GET /auth/signin` MUST continue to present the current sign-in form,
  including hub selection, password visibility toggle, recovery action, and
  redirect handling through `next`.
- `GET /auth/signup` MUST continue to present the current registration form,
  including password confirmation behavior and hub selection.
- `GET /` MUST continue to render the same high-level dashboard for both basic
  and admin users.
- Flash messages MUST remain visible and styled consistently with current
  Bootstrap alerts.
- The admin user filter MUST continue to filter visible rows client-side.
- The admin user modal MUST continue to support viewing a user, editing fields,
  assigning roles, and deleting the user.
- Dropdowns, modals, tooltips, and popovers MUST continue to work with
  Bootstrap behavior.

### 4. Backend Boundary
- Business rules, validation, authorization, and persistence MUST remain in the
  Rust service and repository layers.
- Routes MUST expose serializable DTO or page-model data to React rather than
  leaking domain internals directly into the frontend.
- Async interactions currently driven by HTMX SHOULD move to typed JSON or
  equivalent structured responses. HTML partial compatibility MAY be retained
  temporarily during migration.

### 5. Client Data API Model
- React-owned page initialization MUST prefer typed GET APIs under `/api/v1/`
  rather than HTML-embedded bootstrap payloads.
- The target state SHOULD prefer reusable resource-style APIs over one-off
  page-shaped bootstrap endpoints.
- Shared shell data such as identity, available hubs, and hub-scoped menu
  items SHOULD come from narrower reusable resource endpoints.

### 6. Progressive Enhancement
- Native HTML form submission and redirect flows SHOULD remain the default for
  create/update/delete operations unless a specific interaction benefits from
  asynchronous React behavior.
- The migration SHOULD preserve graceful degradation for critical flows such as
  sign-in, sign-up, logout, and profile updates.

### 7. Frontend Tooling
- The repository MUST gain a supported frontend toolchain for React and
  TypeScript source code.
- Production builds MUST emit versioned static assets that can be referenced by
  the server-rendered layout.
- The `pushkind-auth` server MUST serve the compiled frontend assets directly.
- Local development MUST support efficient iteration for frontend changes
  without requiring manual asset copying on every edit.

## Page Data Requirements
- Each migrated page MUST receive a typed bootstrap payload containing only the
  validated data needed for rendering.
- That transport is transitional only; the long-term target is composition from
  reusable resource-style `/api/v1/...` endpoints instead of page-shaped
  bootstrap payloads.
- Bootstrap payloads SHOULD include:
  current user data, hub/menu lists, flash messages, page identity, and any
  form-select options required by the page.
- Page payloads MUST be derived from DTO-level structs owned by the service or
  route layer.

## Migration Requirements
- The migration MUST be incremental; auth pages, shared layout, basic dashboard,
  and admin dashboard MAY ship in stages.
- Tera templates MAY remain as compatibility wrappers during migration, but
  duplicated markup SHOULD be reduced as React coverage increases.
- Tera MUST be removable as a runtime template dependency once React coverage
  and parity are complete.
- Inline JavaScript and HTMX usage SHOULD be removed only after equivalent React
  behavior is in place.
- Playwright screenshot baselines SHOULD be established before removing the
  corresponding Tera implementation for a migrated page.

## Acceptance Criteria
- Same URLs continue to serve the corresponding pages and actions.
- Visual appearance remains substantially unchanged for sign-in, sign-up,
  navigation, basic dashboard, admin dashboard, and user modal.
- Visual parity is enforced in CI with Playwright screenshot comparisons for
  the migrated UI surface.
- Existing form posts and redirect targets continue to work.
- No backend business rule is moved to the client.
- The React frontend builds reproducibly and its compiled assets are served by
  the application runtime.
- Regression coverage exists for critical frontend behavior and for backend page
  data contracts used by the new frontend.

## Risks
- React markup can drift from the existing templates unless parity is checked
  explicitly.
- Introducing a Node-based frontend toolchain adds build and deployment
  complexity to a previously Rust-centric service.
- Replacing HTMX modal flows may require new structured endpoints or temporary
  dual support for HTML and JSON responses.
