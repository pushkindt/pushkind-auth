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
- Support administrative CRUD for roles, hubs, menus, and users.
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
| GET | `/auth/login` | Reissue session from one-time token (`token` query). |
| POST | `/auth/login` | Login with credentials and issue session JWT. |
| GET | `/auth/signin` | Render sign-in page. |
| GET | `/auth/signup` | Render registration page. |
| POST | `/auth/register` | Register new user. |
| POST | `/auth/recover` | Send password recovery link via email. |
| GET | `/auth/logout` | Logout via shared `pushkind_common` route. |

### Main routes (`/`)
| Method | Path | Description |
| --- | --- | --- |
| GET | `/` | Render dashboard for authenticated user. |
| POST | `/user/save` | Update current user profile. |

### Admin routes (`/admin`)
Admin routes require `SERVICE_ACCESS_ROLE` ("admin") via middleware.

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
- `RequireUserExists` + `RedirectUnauthorized` middleware enforce auth.
- Admin routes require `SERVICE_ACCESS_ROLE` ("admin").
- `next` redirects are validated against `ServerConfig.domain` to prevent
  open redirects.

## Core Flows
### Login
1. Validate `LoginForm` and normalize inputs.
2. `UserReader::login` validates credentials and returns user roles.
3. Build `AuthenticatedUser` claims and issue a JWT.
4. Store the JWT in Actix Identity.

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
- `ServerConfig` fields: `domain`, `database_url`, `address`, `port`,
  `zmq_emailer_pub`, `templates_dir`, `secret`.

## Error Handling
- Repositories return `RepositoryResult<T>` with `RepositoryError`.
- Services return `ServiceResult<T>` and map repository errors into
  `ServiceError`.
- Routes translate service errors into HTTP responses and flash messages.
- Avoid panics; no `unwrap`/`expect` in production paths.

## Data Model (High Level)
- **Hub**: tenant boundary and menu/role owner.
- **User**: belongs to a hub and holds roles.
- **Role**: hub-specific role names.
- **Menu**: hub-specific navigation links.
- Strongly typed value objects (e.g., `UserEmail`, `HubId`, `RoleName`).

## External Integrations
- **pushkind-common**: auth helpers, config models, middleware, and shared routes.
- **pushkind-emailer**: receives recovery emails over ZeroMQ.

## Testing Expectations
- Unit tests for services and forms.
- Use `src/repository/mock.rs` for service isolation.
- Integration tests under `tests/` when DB access is required.

## Operational Notes
- Session cookies use the configured `domain` and are scoped to `.{domain}`.
- `cookie_secure` is `false` in code; must be set to `true` in production.
