# ADR 0003: Introduce Specific Client Data APIs

## Status
Stable

## Context
The Vite static-frontend migration removed backend HTML-shell rendering, but
page initialization is still delivered through route-specific bootstrap
endpoints. That keeps the transport pattern narrowly coupled to individual page
documents and makes shared initialization concerns harder to model cleanly.

The next architectural step is to replace page-shaped bootstrap transport with
more specific client data APIs that reflect actual resources used by the
frontend.

## Decision
- The backend will expose specific client data APIs for React-owned pages.
- New endpoints introduced for this work will be versioned under `/api/v1/`.
- The initial shared GET surface will include:
  hub listing,
  IAM,
  and hub-scoped menu items.
- Existing `GET /api/v1/id` and `GET /api/v1/users` will be assessed for reuse
  before new overlapping endpoints are introduced.
- Static HTML documents will continue to be served by Actix after route-level
  access checks.
- Redirects will remain backend-owned.
- GET APIs will not be used to expose transient alerts.
- `actix_web_flash_messages` is not removed by this ADR alone; flows must first
  migrate to JSON POST request/response handling before that middleware can be
  reduced or removed for them.

## Consequences

### Positive
- Page initialization becomes more reusable and explicit.
- The frontend transport layer becomes less coupled to specific HTML entries.
- The API surface aligns more closely with real resources the frontend uses.
- Existing versioned API capabilities can be reused instead of duplicated where
  the current contracts are already close enough.

### Negative
- The API design can still become too abstract if endpoints stop matching real
  resources.
- Reusing existing endpoints may require careful contract review to avoid
  breaking current API consumers.
- Flash middleware remains necessary for redirect-driven flows until those flows
  migrate to JSON POST handling.
- Migration will temporarily add compatibility layers.

## Rejected Alternatives
- Keep adding route-specific bootstrap endpoints:
  rejected because it scales poorly and duplicates transport concerns.
- Introduce generic shell/page-data endpoints:
  rejected because the abstraction is broader than needed and does not match
  the concrete data the frontend actually needs.
- Remove `actix_web_flash_messages` immediately:
  rejected because it would break redirect-driven feedback until those flows
  migrate to JSON POST handling.
