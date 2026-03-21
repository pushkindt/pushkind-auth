# Plan: React Frontend Migration

## References
- Feature spec:
  [../spec/features/react-frontend-migration.md](../spec/features/react-frontend-migration.md)
- Architecture decision:
  [../spec/decisions/0001-react-islands-frontend.md](../spec/decisions/0001-react-islands-frontend.md)

## Objective
Introduce React for the frontend while preserving the current UI structure,
styling, routes, and backend-owned business logic. The migration remains
server-routed, non-SPA, and ends with Tera removed from the frontend runtime.

## Fixed Implementation Decisions
- Frontend source code WILL live in `frontend/`.
- Production frontend build output WILL live in `assets/dist/`.
- The React toolchain WILL use `npm`, React, TypeScript, and Vite.
- The backend WILL continue to own routing, authentication, authorization,
  validation, redirects, and persistence.
- The application server WILL serve compiled frontend assets directly from the
  existing `/assets` path.
- Tera WILL be used only as a temporary migration wrapper and WILL be removed
  after all frontend pages have been migrated and verified.
- Visual parity in CI WILL use Playwright screenshot comparisons.

## Repository Layout
The implementation MUST create and use the following structure:

```text
frontend/
  package.json
  package-lock.json
  tsconfig.json
  vite.config.ts
  src/
    entries/
    components/
    pages/
    styles/
    lib/
  public/
assets/
  dist/
src/
  dto/
  routes/
  services/
templates/
```

Directory intent:
- `frontend/src/entries/`:
  page entrypoints mounted by server-rendered HTML shells during migration.
- `frontend/src/components/`:
  reusable React UI components.
- `frontend/src/pages/`:
  page-level React components for sign-in, sign-up, dashboard, and admin UI.
- `frontend/src/lib/`:
  shared frontend helpers, typed payload readers, Bootstrap integration hooks.
- `frontend/src/styles/`:
  frontend-owned CSS imports that preserve the existing visual output.
- `assets/dist/`:
  compiled JavaScript, CSS, asset files, and build manifest.

## Toolchain And Build Outputs

### Frontend package management
- Use `npm` as the package manager.
- Commit `frontend/package-lock.json`.
- Do not introduce `pnpm`, `yarn`, or an alternative JavaScript runtime for
  this migration.

### Build tool
- Use Vite to build the React frontend.
- Configure Vite to emit compiled assets into `assets/dist/`.
- Configure Vite to emit a manifest file at `assets/dist/manifest.json`.
- Configure Vite entrypoints explicitly so each server page can load only the
  assets it needs.

### Required `package.json` scripts
The frontend package MUST expose at least these scripts:
- `dev`
- `build`
- `preview`
- `test`
- `lint`
- `typecheck`
- `playwright:screenshots`

### Source control hygiene
- Add `frontend/node_modules/` to `.gitignore`.
- Add `assets/dist/` to `.gitignore` unless deployment requires committed build
  artifacts. If that deployment requirement exists later, it must be documented
  explicitly before changing this rule.

## Backend Integration

### Asset serving
- Add explicit Actix static file serving for `/assets` if it is not already
  implemented in application code.
- The server MUST serve files from the repository `assets/` directory, which
  includes `assets/dist/`.

### Asset manifest loading
- Add a backend helper that reads `assets/dist/manifest.json` and resolves the
  generated JS and CSS filenames for each page entrypoint.
- The helper MUST fail clearly during startup or request rendering if the build
  manifest is required but missing.

### Page bootstrap payloads
- Introduce typed DTOs for React page bootstrap data under `src/dto/`.
- Do not pass domain objects directly into template or React payload contexts.
- Define separate payload structs for:
  sign-in,
  sign-up,
  basic dashboard,
  admin dashboard,
  user modal,
  shared shell data.

### Server-rendered shell during migration
- During migration, the backend MAY render a minimal HTML shell that:
  includes the compiled React entrypoint,
  includes the compiled CSS,
  embeds the page bootstrap payload,
  provides the mount node for React.
- That shell MUST NOT contain duplicated page markup once a page is owned by
  React.

## Frontend Runtime Requirements

### Bootstrap integration
- Keep Bootstrap CSS and Bootstrap Icons in the rendered output.
- Preserve Bootstrap JavaScript behavior for modals, dropdowns, popovers, and
  tooltips.
- Wrap Bootstrap imperative APIs behind React-safe helpers in
  `frontend/src/lib/`.

### Data bootstrapping
- The server MUST embed page bootstrap payloads as JSON in the HTML response.
- React entrypoints MUST read the payload from the DOM, validate its shape, and
  render the corresponding page.

### Form handling
- Native HTML form submission remains the default.
- React MAY enhance form behavior client-side, but MUST NOT take over backend
  validation, redirect logic, or authorization checks.

## Migration Sequence

### Phase 1: Foundation
Deliverables:
- `frontend/` directory with React, TypeScript, and Vite configured.
- Build output emitted to `assets/dist/`.
- Actix static serving for `/assets`.
- Manifest loading helper on the backend.
- Shared React shell capable of mounting one page.
- Developer documentation for:
  installing Node via `mise`,
  running backend and frontend together,
  building production assets.

Exit criteria:
- `npm run build` succeeds.
- The application can render one React-backed page while serving assets from
  `assets/dist/`.

### Phase 2: Shared Shell
Deliverables:
- Shared React shell for flash messages and common page wiring.
- Common payload reader and shared layout helpers.
- Bootstrap lifecycle integration moved out of inline scripts.

Exit criteria:
- No inline JavaScript remains in the base layout for shared shell behavior
  that React now owns.

### Phase 3: Auth Pages
Deliverables:
- React sign-in page.
- React sign-up page.
- Password visibility toggle migrated to React.
- Password confirmation behavior migrated to React.
- Recovery flow triggered from React while preserving backend route behavior.

Exit criteria:
- `/auth/signin` and `/auth/signup` are visually equivalent to the current
  pages.
- Existing form posts and `next` redirect behavior still work.
- Playwright screenshot baselines exist for both pages.

### Phase 4: Basic Dashboard
Deliverables:
- React rendering for the non-admin home page.
- React-controlled profile form interactions where applicable.
- Preservation of existing POST `/user/save` behavior.

Exit criteria:
- `/` for non-admin users is React-rendered.
- Visual parity is verified by Playwright screenshots.

### Phase 5: Admin Dashboard
Deliverables:
- React rendering for roles, hubs, menu items, and user list sections.
- React-controlled client-side filtering.
- React user modal with typed payload loading.
- Replacement for HTMX user modal flow.

Exit criteria:
- `/` for admin users is React-rendered.
- User modal editing and delete flows work without HTMX.
- Playwright screenshot baselines exist for admin dashboard states and user
  modal states.

### Phase 6: Tera Removal
Deliverables:
- Remove obsolete page templates and template fragments that are no longer used
  for frontend rendering.
- Remove HTMX dependency from rendered pages.
- Remove inline page-specific scripts no longer needed.
- Remove Tera as a frontend rendering dependency once no user-facing page
  requires it.

Exit criteria:
- No user-facing page depends on Tera for page markup generation.
- React owns all frontend page markup.
- The server still serves the same URLs with the same backend semantics.

## Testing And Verification

### Frontend verification
- Add component tests for local interactive behavior.
- Add type-checking as a required verification step.
- Add linting as a required verification step.

### Visual parity verification
- Use Playwright screenshot comparisons as the primary regression mechanism.
- Capture baseline screenshots for:
  `/auth/signin`,
  `/auth/signup`,
  `/` as non-admin,
  `/` as admin,
  admin user modal.
- Capture screenshots at deterministic viewport sizes for desktop and mobile.
- Stabilize test data, fonts, and timing so screenshots are reproducible in CI.

### Backend verification
- Add integration coverage for bootstrap payload generation.
- Add integration coverage for manifest loading and asset tag generation if that
  logic is introduced.
- Preserve or extend existing backend route tests where page behavior changes.

### Required commands
- `cargo build --all-features --verbose`
- `cargo test --all-features --verbose`
- `cargo clippy --all-features --tests -- -Dwarnings`
- `cargo fmt --all -- --check`
- `cd frontend && npm run typecheck`
- `cd frontend && npm run test`
- `cd frontend && npm run build`
- `cd frontend && npm run playwright:screenshots`

## CI Requirements
- CI MUST install both Rust and Node toolchains.
- CI MUST build frontend assets before running screenshot-based UI checks.
- CI MUST fail when Playwright screenshot diffs are detected.
- CI MUST keep screenshot baselines versioned in the repository if Playwright is
  configured for committed baselines.

## Risks And Mitigations
- Markup drift:
  define Playwright screenshot baselines before removing the old
  implementation.
- Asset pipeline complexity:
  keep the toolchain limited to npm, Vite, React, and TypeScript.
- Backend/frontend contract drift:
  use typed DTOs for all page bootstrap payloads.
- Incomplete migration:
  do not remove Tera until every user-facing page has a React replacement and
  screenshot parity coverage.

## Definition Of Done
- Frontend source lives in `frontend/` and nowhere under `src/`.
- Production frontend assets are emitted to `assets/dist/`.
- The application server serves the compiled assets directly.
- Sign-in, sign-up, basic dashboard, admin dashboard, and user modal are React
  rendered.
- Playwright screenshot comparisons protect the migrated UI in CI.
- HTMX and page-specific inline scripts are removed.
- Tera is no longer required for frontend page rendering.
