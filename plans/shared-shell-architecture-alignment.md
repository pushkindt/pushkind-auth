# Plan: Shared Shell Architecture Alignment

## References
- Service baseline:
  [../SPEC.md](../SPEC.md)
- Existing React migration spec:
  [../specs/features/react-frontend-migration.md](../specs/features/react-frontend-migration.md)
- Feature spec:
  [../specs/features/shared-shell-architecture-alignment.md](../specs/features/shared-shell-architecture-alignment.md)

## Objective
Refactor `pushkind-auth` onto the same React shell architecture used by
`pushkind-crm`, `pushkind-emailer`, `pushkind-todo`, and `pushkind-orders`
without changing auth routes, auth flows, or visible UI structure. The end
state is shared navbar, shell, flash, fatal-state, no-access, and shell data
integration through `pushkind-common/frontend`.

## Fixed Implementation Decisions
- Shared frontend shell code WILL be consumed from
  `@pushkind/frontend-shell`.
- `pushkind-auth` WILL remain server-routed and non-SPA.
- Existing auth/admin page markup and Bootstrap styling MUST remain visually
  equivalent.
- Shared abstractions MUST be adopted only where the auth UI can preserve the
  existing rendered structure.
- Resource-specific parsing and auth-page-specific interactions MAY remain
  local if they are not generic shell concerns.

## Migration Sequence

### Phase 1: Baseline Audit
Deliverables:
- Inventory of local auth shell pieces still duplicating shared code.
- Mapping from current auth components to shared package modules.
- Identification of auth-specific behavior that must remain local.

Exit criteria:
- The cutover scope is explicit and limited to shell architecture concerns.

### Phase 2: Navbar Alignment
Deliverables:
- Replace the local auth navbar implementation with the shared navbar
  primitive.
- Preserve auth brand, active-link logic, and current menu ordering.
- Keep existing Bootstrap markup and visual parity.

Exit criteria:
- Auth navigation renders through the shared navbar component without visible
  drift.

### Phase 3: Flash And Fatal-State Alignment
Deliverables:
- Align auth flash presentation with the common shared shell flash behavior.
- Replace local fatal shell state rendering with the shared fatal-state
  primitive.
- Preserve current auth copy and page framing.

Exit criteria:
- Async React actions in auth surface messages through the shared shell flash
  behavior.
- Shell failures render through the shared fatal-state primitive.

### Phase 4: Shell Wrapper And No-Access Alignment
Deliverables:
- Adopt the shared shell wrapper architecture where auth page framing matches
  the common model.
- Align any no-access or shell-denied pages to the shared no-access pattern if
  applicable.
- Remove auth-local shell duplication made obsolete by the shared package.

Exit criteria:
- Auth shell wiring no longer carries private copies of navbar/flash/fatal/no-
  access infrastructure that now exists in `pushkind-common/frontend`.

### Phase 5: Shared Shell Data And Type Alignment
Deliverables:
- Switch auth shell loading to shared shell API helpers where contracts match.
- Replace remaining duplicated shell base types with imports or aliases from
  the shared package.
- Keep resource-specific auth/admin parsing local.

Exit criteria:
- Auth no longer duplicates shell-level frontend parsing and typing that is now
  shared.

### Phase 6: Cleanup And Verification
Deliverables:
- Remove obsolete local auth shell helpers and components replaced by the
  shared package.
- Update docs if the shared shell architecture is now part of the service
  baseline.
- Run frontend verification and compare visual output against the current auth
  UI.

Exit criteria:
- `pushkind-auth` is on the same shell architecture as the other migrated
  services.
- Remaining local code is auth-specific rather than shell-generic.

## Verification
- `npm run format`
- `npm run lint`
- `npm run test`
- additional visual verification for sign-in, sign-up, and admin/index states
  against current UI
