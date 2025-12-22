# pushkind-auth Specification

## Purpose
`pushkind-auth` is the Pushkind single sign-on service. It authenticates users,
issues session JWTs, exposes hub-scoped user APIs, and provides admin tooling for
roles, hubs, menus, and user management.

## Goals
- Provide hub-aware login and session management.
- Keep business logic in service layer with thin handlers and repositories.
- Maintain type-safe domain models with validated, normalized inputs at the
  boundaries.
- Support administrative management for roles, hubs, menus, and users.
- Expose JSON APIs for hub user directory queries.
- Offer a password recovery flow via email.

## Non-Goals
- OAuth or third-party identity providers.
- Multi-database support beyond SQLite.
- Client-side rendering or SPA behavior.

## Architecture Overview
- **Routes (`src/routes`)**: Actix handlers extract inputs, call services, and
  render templates or return JSON.
- **Services (`src/services`)**: Application use-cases; convert forms to domain
  types, enforce rules, and return DTOs/errors.
- **Repository (`src/repository`)**: Traits + Diesel-backed implementation for
  SQLite; convert Diesel models to domain types.
- **Domain (`src/domain`)**: Strongly typed models (e.g., `UserEmail`, `HubId`,
  `RoleName`, `MenuName`) that assume validated inputs.
- **Forms (`src/forms`)**: `validator`-backed structs for input validation and
  normalization.
- **DTOs (`src/dto`)**: API/template-facing shapes.
- **Templates (`templates/`)**: Server-rendered UI via Tera.

## Runtime Components
- Actix Web server configured in `src/lib.rs::run`.
- Diesel SQLite pool via `pushkind_common::db::establish_connection_pool`.
- ZeroMQ publisher for email recovery notifications.
- Cookie-based sessions + Actix Identity for authentication and flash messages.

## HTTP Routes
All paths are mounted under scopes configured in `src/lib.rs`.

### Auth routes (`/auth`)
| Method | Path | Description |
| --- | --- | --- |
| GET | `/auth/login` | Reissue session from short-lived token (`token` query). |
| POST | `/auth/login` | Login with credentials and issue session JWT. |
| GET | `/auth/signin` | Render sign-in page. |
| GET | `/auth/signup` | Render registration page. |
| POST | `/auth/register` | Register new user. |
| POST | `/auth/recover` | Send password recovery link via email. |
| POST | `/auth/logout` | Logout via shared `pushkind_common` route. |

### Main routes (`/`)
| Method | Path | Description |
| --- | --- | --- |
| GET | `/` | Render dashboard for authenticated user. |
| POST | `/user/save` | Update current user profile. |

### Admin routes (`/admin`)
Admin routes MUST require `SERVICE_ACCESS_ROLE` ("admin") and enforce it via
service-layer authorization checks (`pushkind_common::routes::ensure_role`).

| Method | Path | Description |
| --- | --- | --- |
| POST | `/admin/role/add` | Create a role. |
| POST | `/admin/role/delete/{role_id}` | Delete a role. |
| POST | `/admin/user/modal/{user_id}` | Render user modal body. |
| POST | `/admin/user/delete/{user_id}` | Delete a user. |
| POST | `/admin/user/update/{user_id}` | Update user profile and roles. |
| POST | `/admin/hub/add` | Create a hub. |
| POST | `/admin/hub/delete/{hub_id}` | Delete a hub. |
| POST | `/admin/menu/add` | Create a menu item. |
| POST | `/admin/menu/delete/{menu_id}` | Delete a menu item. |

### API routes (`/api`)
| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/v1/id` | Get current user or a user by `id` query param. |
| GET | `/api/v1/users` | List users for the current hub with filters. |

## Authentication and Authorization
- Actix Identity stores a JWT token issued from `AuthenticatedUser`.
- `/` and `/admin` scopes MUST use `RequireUserExists` +
  `RedirectUnauthorized` middleware to enforce authentication.
- API routes MUST require `AuthenticatedUser` extraction and return `401` when
  the session token is missing or invalid.
- Admin routes MUST require `SERVICE_ACCESS_ROLE` ("admin") and enforce it via
  service-layer authorization checks (`ensure_role`).
- `next` redirects are validated against `ServerConfig.domain` to prevent
  open redirects.
- Role names are case-sensitive; `SERVICE_ACCESS_ROLE` is a fixed constant, not
  a runtime configuration.
- Admin authorization is global (role-based), but some operations are hub-scoped:
  users and menus are restricted to the admin's hub; roles and hubs are global.
- Admins MAY manage other admins; restrictions:
  users MUST NOT delete themselves, admins MUST NOT delete their own hub, and
  the base admin role (id `1`) MUST NOT be deleted.

## Core Flows
### Login
1. Validate `LoginForm` and normalize inputs.
2. `UserReader::login` validates credentials and returns user roles.
3. Build `AuthenticatedUser` claims and issue a JWT.
4. Store the JWT in Actix Identity.

### JWT Claims
- `sub`: user id as a string.
- `email`: user email (lower-cased).
- `hub_id`: hub id.
- `name`: user display name (may be empty).
- `roles`: array of role names.
- `exp`: unix timestamp (seconds).
- Session JWTs MUST set `exp` to now + 7 days; recovery link JWTs MUST set `exp`
  to now + 1 day.

### Recovery
1. Validate `RecoverForm` inputs.
2. Load user by email/hub.
3. Issue a 1-day JWT and build `/auth/login?token=...` URL.
4. Send ZMQ message to emailer service.

### Registration
1. Validate `RegisterForm`.
2. Create user via repository.

## Configuration
- Config is loaded from `config/default.yaml`, then `config/{APP_ENV}.yaml`,
  then `APP_` environment variables.
- `ServerConfig` fields are required with no defaults: `domain`, `database_url`,
  `address`, `port`, `zmq_emailer_pub`, `templates_dir`, `secret`.
- Missing or invalid configuration causes startup to log an error and exit
  with status code `1`.

## Error Handling
- Repositories MUST return `RepositoryResult<T>` with `RepositoryError`.
- Services MUST return `ServiceResult<T>` and map repository errors into
  `ServiceError`.
- Server-rendered routes MUST translate service errors into HTTP responses;
  validation and authorization failures MUST use redirects (303) and flash
  messages.
- API routes MUST translate service errors into HTTP responses.
- Production paths MUST NOT use `unwrap` or `expect`.

## HTTP Error Semantics
Server-rendered routes MUST use `303 See Other` redirects plus flash messages
for most failures. API routes MUST return JSON on success; on error they MUST
return the status codes below and SHOULD return an empty body unless otherwise
noted. Flash message content MUST NOT be treated as API-stable.

| Condition | HTTP | Notes |
| --- | --- | --- |
| Invalid credentials (`POST /auth/login`) | 303 | Redirect to `/auth/signin` with error flash. |
| Registration conflict (duplicate email in hub) | 303 | Redirect to `/auth/signup` with error flash. |
| Recovery for non-existent user | 303 | Redirect to `/auth/signin` with error flash. |
| Validation error (HTML forms) | 303 | Redirect to form page with error flash. |
| Unauthenticated request under `/` or `/admin` | 303 | Redirect to `/auth/signin?next=...` (via `RedirectUnauthorized`). |
| Missing admin role under `/admin/*` | 303 | Redirect to `/` with error flash (most endpoints). |
| API user not found (`GET /api/v1/id`) | 404 | Empty body. |
| API internal error (`/api/*`) | 500 | Empty body. |
| Internal service failures (HTML routes) | 500 | No user-visible detail beyond flash (if set). |

## Data Model (High Level)
- **Hub**: tenant boundary and menu owner.
- **User**: belongs to a hub and holds roles.
- **Role**: global role names assigned to users.
- **Menu**: hub-specific navigation links.
- Strongly typed value objects (e.g., `UserEmail`, `HubId`, `RoleName`).

## Invariants
- A User belongs to exactly one Hub.
- User email uniqueness is enforced per Hub (`UNIQUE(email, hub_id)`).
- Hub names are globally unique.
- Role names are globally unique; role lookup and authorization are case-sensitive.
- The base admin role has id `1` and cannot be deleted.
- Users may exist without any roles.
- User-role assignments are unique per `(user_id, role_id)` and are removed when
  either the user or role is deleted.
- Menu entries belong to exactly one Hub.
- Deleting a Hub MUST delete its users, their role assignments, and its menu
  entries.

## External Integrations
- **pushkind-common**: auth helpers, config models, middleware, and shared routes.
- **pushkind-emailer**: receives recovery emails over ZeroMQ.

## Contributor Notes
Contributor guidance, including testing expectations, lives in
`CONTRIBUTING.md`.

## Operational Notes
- Session cookies use the configured `domain` and are scoped to `.{domain}`.
- `cookie_secure` is `false` in code; must be set to `true` in production.
