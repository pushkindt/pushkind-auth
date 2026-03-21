# Plan: Vite Static Frontend Target

## References
- Feature spec:
  [../spec/features/vite-static-frontend.md](../spec/features/vite-static-frontend.md)
- Architecture decision:
  [../spec/decisions/0002-vite-static-frontend-documents.md](../spec/decisions/0002-vite-static-frontend-documents.md)

## Objective
Replace Rust-owned frontend HTML document rendering with Vite-built static HTML
documents, while keeping backend routing, auth, redirects, validation, and
JSON data contracts under Rust control.

## Fixed Implementation Decisions
- The frontend source code WILL remain in `frontend/`.
- Production build output WILL remain in `assets/dist/`.
- Vite WILL build HTML entry files in addition to hashed JS and CSS assets.
- Actix WILL keep owning access checks and route dispatch before selecting which
  built HTML document to serve.
- Bootstrap data WILL be fetched from typed JSON endpoints, not embedded in the
  HTML response.
- React WILL remain non-SPA and route-specific.

## Repository Layout
The implementation SHOULD converge on:

```text
frontend/
  auth/
    signin.html
    signup.html
  app/
    index-basic.html
    index-admin.html
  src/
    entries/
    bootstrap/
    components/
    pages/
    styles/
assets/
  dist/
    auth/
    app/
    assets/
src/
  dto/
  routes/
  services/
```

## Implementation Sequence

### Phase 1: Static HTML Entry Files
Deliverables:
- Add Vite-managed HTML files for sign-in and sign-up.
- Ensure those files include the correct React entrypoints and shared assets.
- Keep Rust shell rendering temporarily for dashboard pages.

Exit criteria:
- Vite emits built HTML files for auth pages into `assets/dist/`.

### Phase 2: Auth Bootstrap Endpoints
Deliverables:
- Add typed JSON endpoints for sign-in and sign-up bootstrap data.
- Update auth React entrypoints to fetch bootstrap data after load.
- Update auth GET routes to serve built HTML files after access checks.

Exit criteria:
- `/auth/signin` and `/auth/signup` no longer depend on Rust HTML rendering.

### Phase 3: Dashboard Static HTML
Deliverables:
- Add separate built HTML entries for basic and admin dashboards.
- Keep route-level selection in Rust based on authenticated role.
- Add typed bootstrap endpoints for both dashboard variants.

Exit criteria:
- `GET /` serves built HTML documents for both user types.

### Phase 4: Remove Rust Shell Rendering
Deliverables:
- Delete `render_frontend_page` or equivalent HTML string assembly helpers.
- Remove any remaining backend HTML-shell logic for React-owned pages.
- Ensure all frontend pages use built HTML plus JSON bootstrap.

Exit criteria:
- No Rust code emits the frontend document shell.

## Backend Work Items
- Add helpers to serve specific built HTML files from `assets/dist/`.
- Add DTO-backed JSON bootstrap endpoints.
- Preserve redirects for unauthenticated and unauthorized requests.
- Keep POST handlers unchanged unless a typed JSON contract is explicitly
  required.

## Frontend Work Items
- Create HTML entry files and align them with route ownership.
- Implement a shared page bootstrap loader.
- Add loading and fatal error states for bootstrap fetches.
- Ensure CSS and icon dependencies remain bundled and deterministic.

## Verification
- `cargo build --all-features --verbose`
- `cargo test --all-features --verbose`
- `cargo clippy --all-features --tests -- -Dwarnings`
- `cargo fmt --all -- --check`
- `cd frontend && npm run typecheck`
- `cd frontend && npm run build`
- Playwright screenshot parity for auth and dashboard pages after each phase
