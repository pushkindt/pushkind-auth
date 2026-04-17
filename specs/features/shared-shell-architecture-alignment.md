# Shared Shell Architecture Alignment

## Status
Stable

## Date
2026-04-13

## Summary
Align `pushkind-auth` with the shared React shell architecture already used by
`pushkind-crm`, `pushkind-emailer`, `pushkind-todo`, and `pushkind-orders`.
This work MUST preserve the existing auth UI and route behavior while
converging on shared navbar, shell, flash, fatal-state, no-access, and shell
data patterns owned by `pushkind-common/frontend`.

## Problem
`pushkind-auth` was migrated to React earlier than the other services and does
not currently use the same shell abstractions. That makes the service an
outlier in navigation, flash presentation, fatal-state handling, and frontend
code sharing, which blocks consistent cross-service UX and limits what can be
centralized in `pushkind-common`.

## Goals
- Converge `pushkind-auth` on the same shell architecture as the other migrated
  services.
- Preserve existing auth and admin UI markup, Bootstrap classes, and user
  visible copy.
- Keep the application server-routed and non-SPA.
- Reuse shared frontend shell modules from `@pushkind/frontend-shell`.
- Make navbar, shell, flash, fatal-state, no-access, and shell data handling
  consistent across services.

## Non-Goals
- Redesigning sign-in, sign-up, or admin pages.
- Changing auth routes, session behavior, redirects, or business logic.
- Moving validation or authorization logic into the frontend.
- Reworking resource-specific page data beyond what shell alignment requires.

## In Scope
- Shared navigation and user-menu architecture.
- Shared flash message delivery behavior.
- Shared shell wrapper and fatal-state pattern.
- Shared no-access page pattern if applicable to auth-owned surfaces.
- Shared shell data and hub-menu loading helpers.
- Shared frontend types for shell and mutation primitives where the auth
  frontend uses the same contracts.

## Out Of Scope
- New frontend pages or admin features.
- Rust service-layer refactors unrelated to shell delivery.
- Cross-service UX changes beyond converging on the established shell pattern.

## Functional Requirements

### 1. Shell Convergence
- `pushkind-auth` MUST use the same shared shell architecture primitives as the
  other migrated services where the contracts now match.
- Shared primitives SHOULD come from `pushkind-common/frontend`.
- The resulting auth shell MUST preserve current visual output and route
  behavior.

### 2. Navbar Convergence
- Auth navigation MUST converge on the shared navbar component shape.
- Differences such as service label, active-link handling, and auth-specific
  menu content MUST be expressed through props rather than a separate navbar
  implementation.
- Existing navbar HTML/CSS semantics MUST remain visually equivalent.

### 3. Flash Behavior
- Flash presentation MUST converge on the same behavior used by the other
  migrated services.
- React-owned async actions MUST continue to surface messages through the
  shared shell flash mechanism.
- Existing Bootstrap alert styling MUST remain intact.

### 4. Fatal State And No-Access Handling
- Fatal shell failures MUST render through the shared fatal-state primitive.
- If auth-owned no-access pages or equivalent shell-denied pages exist, they
  MUST use the shared no-access page architecture.
- Service-specific copy MAY remain if the shared primitives support it through
  props.

### 5. Shell Data APIs
- Shared shell data loading MUST use the same frontend helper model as the
  other migrated services.
- `pushkind-auth` MUST rely on shared shell parsing/fetch helpers where the
  payload contracts match.
- Service-specific page/resource parsing MAY remain local.

### 6. UI Preservation
- Existing auth page and admin page UI MUST remain substantially unchanged.
- Markup drift is not acceptable merely to make the shell abstractions fit.
- Any deviation required by the shared architecture MUST be explicitly
  documented before implementation.

## Acceptance Criteria
- `pushkind-auth` uses the shared navbar architecture.
- `pushkind-auth` uses the shared flash delivery pattern.
- `pushkind-auth` uses shared fatal-state and shell helpers where contracts
  match.
- The auth frontend imports shared shell modules from
  `@pushkind/frontend-shell` instead of keeping local duplicates.
- Existing routes, redirects, and auth flows still behave the same.
- Visual output remains substantially unchanged.

## Risks
- Auth pages are more auth-flow-oriented than the other service shells, so
  naive convergence can cause unnecessary markup drift.
- Flash and fatal-state alignment can accidentally change page framing if
  service-specific copy or wrappers are lost.
