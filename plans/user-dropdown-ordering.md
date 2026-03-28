# Plan: User Dropdown Ordering

## References
- Feature spec:
  [../specs/features/user-dropdown-ordering.md](../specs/features/user-dropdown-ordering.md)

## Objective
Make dropdown ordering explicit in the React component contract so local items
render first, fetched hub menu items render next, and logout stays last.

## Fixed Implementation Decisions
- The change WILL stay within the React frontend.
- `UserMenuDropdown` WILL receive separate local and fetched item arrays.
- `Navigation` WILL provide the local home entry explicitly.
- A frontend regression test WILL assert the rendered ordering.
- No ADR is required because this does not change the project architecture.

## Implementation Sequence

### Phase 1: Component Contract
Deliverables:
- Update `frontend/src/components/UserMenuDropdown.tsx` to accept local and
  fetched item groups explicitly.
- Render local items before fetched items and keep logout last.

Exit criteria:
- Dropdown ordering is encoded by props rather than implicit rendering logic.

### Phase 2: Navigation Wiring
Deliverables:
- Update `frontend/src/components/Navigation.tsx` to pass the local home item
  separately from fetched hub menu items.

Exit criteria:
- The dashboard pages continue to use the existing fetched menu payload for both
  navbar links and dropdown links, with explicit local-item ordering in the
  dropdown.

### Phase 3: Verification
Deliverables:
- Add a frontend test that asserts local items render before fetched items and
  logout remains last.
- Run targeted frontend test and typecheck commands.

Exit criteria:
- The ordering is covered by a repeatable frontend test.
