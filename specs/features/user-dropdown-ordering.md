# User Dropdown Ordering

## Status
Stable

## Date
2026-03-28

## Summary
Make the React user dropdown render local navigation entries before hub menu
entries fetched from the menu API, while keeping logout as the last action.

## Problem
The dropdown component currently mixes a hard-coded local home link with fetched
menu items through an implicit prop shape. That makes the intended ordering
fragile and does not scale cleanly when more local dropdown actions are added.

## Goals
- Make local dropdown items explicit in the component contract.
- Render local dropdown items before fetched menu API items.
- Keep the logout action as the final dropdown item.

## Non-Goals
- Changing the top navbar ordering.
- Changing menu API payloads or backend ordering semantics.
- Changing the visual design of the dropdown beyond the required item order.

## Functional Requirements

### 1. Explicit Dropdown Sections
- The React dropdown component MUST accept local items separately from fetched
  menu items.
- Local items MUST be rendered before fetched menu items.

### 2. Stable Logout Placement
- The logout action MUST always render after all local and fetched navigation
  items.

### 3. Regression Coverage
- Frontend tests MUST verify the rendered dropdown order for local items,
  fetched items, and logout.

## Acceptance Criteria
- The user dropdown renders local items first.
- Hub menu items fetched from `/api/v1/hubs/{hub_id}/menu-items` render after
  the local items.
- The logout action remains the last dropdown entry.
- A frontend test fails if logout moves ahead of fetched or local items.
