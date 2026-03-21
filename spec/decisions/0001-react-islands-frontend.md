# ADR 0001: Adopt React Incrementally Without SPA Routing

## Status
Proposed

## Context
`pushkind-auth` currently renders its UI with Tera templates and augments that
markup with inline JavaScript and HTMX. The desired change is to use React for
frontend development while preserving the existing style, markup, routes, and
backend-controlled flows.

`SPEC.md` explicitly lists client-side rendering and SPA behavior as non-goals.
That means a React migration cannot be framed as a full client-routed rewrite
without first changing the system specification.

## Decision
- Keep Actix routes and server-side request handling as the source of truth for
  navigation, redirects, authentication, and authorization.
- Introduce React incrementally as the frontend component layer on the existing
  URLs.
- Do not introduce client-side routing.
- Serve compiled frontend assets directly from `pushkind-auth`.
- Keep Tera only as a migration wrapper until React coverage is complete and
  parity is proven, then remove it.
- Move frontend data exchange to typed page models and structured async
  responses rather than embedding more ad hoc JavaScript behavior in templates.
- Replace HTMX and inline scripts only when an equivalent React interaction has
  been shipped.
- Use Playwright screenshot comparisons as the primary CI mechanism for visual
  parity during the migration.

## Consequences

### Positive
- React can be adopted without rewriting the backend architecture.
- The migration can proceed page by page with lower release risk.
- Visual parity is easier to preserve because route shape and server ownership
  do not change.
- The end state is a single frontend rendering model instead of a permanent
  React-plus-Tera hybrid.

### Negative
- The system will temporarily carry both Tera and React concerns during the
  migration.
- A Node-based frontend toolchain becomes part of build and deployment.
- Some endpoints may need dual support during the transition.

## Rejected Alternatives
- Full SPA rewrite:
  rejected because it conflicts with the current specification and would widen
  the scope from frontend implementation migration to full application
  architecture change.
- Keep HTMX and inline JavaScript:
  rejected because it does not satisfy the goal of moving to React.

## Follow-Up
If the team later wants React to own full page rendering or client-side
navigation, that must be captured in a new ADR and corresponding `SPEC.md`
changes.
