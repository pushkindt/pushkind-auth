# Vite Static Frontend Target

## Status
Stable

## Date
2026-03-27

## Summary
Move the frontend runtime to Vite-built static HTML, JavaScript, and CSS so
`pushkind-auth` no longer assembles HTML documents in Rust. Rust MUST continue
to own authentication, authorization, redirects, validation, persistence, and
JSON/bootstrap endpoints, while React and Vite own the browser document and all
page markup.

## Problem
The current React migration removed Tera, but Rust still builds the HTML shell
at request time. That keeps the backend on the rendering path for document
markup and leaves the system with an awkward split where React owns the page UI
but Rust still owns the document string.

## Goals
- Make Vite the owner of the frontend HTML documents.
- Remove request-time HTML document construction from Rust.
- Keep the same route URLs and backend semantics.
- Keep the application server responsible for serving built frontend assets.
- Move page bootstrap delivery to typed JSON endpoints instead of embedded JSON
  inside server-generated HTML.

## Non-Goals
- Introducing SPA routing.
- Moving business logic, validation, authorization, or persistence into the
  frontend.
- Replacing Bootstrap or redesigning the existing UI.
- Introducing a separate Node deployment service for SSR.

## Desired End State
- Vite builds static HTML entry documents for:
  `/auth/signin`,
  `/auth/signup`,
  `/`,
  and any future React-owned pages.
- Actix serves those built HTML documents directly from `assets/dist/`.
- React starts from those static documents and fetches typed bootstrap data from
  backend JSON endpoints.
- Rust no longer contains inline HTML document templates for frontend pages.

## Functional Requirements

### 1. Document Ownership
- The browser HTML document for each React page MUST be authored in the
  frontend workspace and built by Vite.
- Rust MUST NOT assemble `<html>`, `<head>`, `<body>`, script tags, or mount
  node markup for those pages at request time.
- The built HTML documents MUST preserve the current Bootstrap, Bootstrap
  Icons, favicon, and React entry loading requirements.

### 2. Route Model
- Existing public URLs MUST remain unchanged.
- `GET /auth/signin`, `GET /auth/signup`, and `GET /` MUST continue to be
  backend routes for authentication and authorization purposes, but those
  routes SHOULD respond by serving the matching built HTML file.
- Backend redirects and access checks MUST still happen before serving the HTML
  document.

### 3. Data Loading
- Page bootstrap data MUST move from embedded JSON in HTML to typed backend
  endpoints.
- Each React page MUST load only the bootstrap data it needs from a dedicated
  backend endpoint or structured page-data endpoint.
- Bootstrap endpoints MUST return DTO-level structs only.
- Admin modal and other async flows MUST continue to use structured JSON.

### 4. Frontend Build Outputs
- Vite MUST produce static HTML files as build artifacts in `assets/dist/`.
- Vite MUST continue to emit hashed JS and CSS assets for cache safety.
- The application server MUST continue to serve all built frontend files from
  `/assets`.

### 5. Progressive Migration
- The transition MAY happen page by page.
- During migration, some pages MAY still use backend-served shells while others
  use Vite-built HTML, but the target state MUST eliminate Rust-owned frontend
  document rendering completely.
- Any temporary hybrid behavior MUST be documented in the plan.

## Backend Requirements
- Add explicit helpers for serving built HTML entry files from `assets/dist/`.
- Add typed bootstrap endpoints for:
  sign-in,
  sign-up,
  basic dashboard,
  admin dashboard.
- Preserve existing POST form actions and redirect behavior.
- Keep modal and async admin endpoints typed and structured.

## Frontend Requirements
- Add Vite HTML entry files for each React-owned page.
- Add a shared frontend mechanism for fetching bootstrap data before initial
  render.
- Handle loading and bootstrap-fetch failure states explicitly.
- Keep Bootstrap JavaScript integration working without backend HTML assembly.

## Acceptance Criteria
- No Rust code constructs frontend HTML documents at request time.
- No page bootstrap payload is embedded into server-generated HTML.
- The same URLs continue to work with the same backend redirect and auth rules.
- React still owns all page markup.
- Built HTML, JS, and CSS are all served by `pushkind-auth`.

## Risks
- Initial page render will depend on an extra bootstrap fetch unless data is
  prefetched another way.
- Serving static HTML while preserving access control requires route-level file
  selection instead of fully public file serving.
- Migration will require explicit handling for loading, unauthorized, and fetch
  error states that were previously hidden by server-side shell rendering.
