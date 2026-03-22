# ADR 0002: Let Vite Own Frontend Documents

## Status
Stable

## Context
The React migration removed Tera, but the backend still constructs the HTML
document shell in Rust. That keeps the frontend document under backend control
even though React already owns the page markup and Vite already owns the asset
pipeline.

The cleaner target is for Vite to build the HTML documents as static frontend
artifacts, while Rust continues to own route access checks and JSON data.

## Decision
- Vite will own the HTML documents for React-rendered pages.
- Rust will serve those built HTML files after applying route-level auth and
  authorization checks.
- Page bootstrap data will move to typed JSON endpoints instead of embedded JSON
  in server-generated HTML.
- React will remain route-based and non-SPA.
- No Node SSR runtime will be introduced.

## Consequences

### Positive
- Frontend document markup lives with the frontend build instead of Rust string
  assembly.
- The asset pipeline becomes conceptually cleaner: Vite owns HTML, JS, and CSS.
- Rust remains focused on access control, redirects, APIs, and business logic.

### Negative
- Initial render now needs a bootstrap fetch unless another prefetch strategy is
  adopted.
- More typed page-data endpoints are required.
- Route handlers must map authenticated requests to the correct built HTML file.

## Rejected Alternatives
- Keep Rust-generated HTML shells:
  rejected because it leaves frontend document ownership split across Rust and
  React.
- Introduce React SSR:
  rejected because it adds a second runtime and deployment path without solving
  a current backend need.
