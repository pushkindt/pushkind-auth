# pushkind-auth

`pushkind-auth` is a single sign-on (SSO) authentication service that powers
the Pushkind ecosystem. It provides session and token based authentication, user management and administrative endpoints for other Pushkind services. The project
is implemented in Rust on top of Actix Web, Diesel, and Tera and integrates
tightly with the shared `pushkind-common` crate for authentication,
configuration, and reusable UI helpers.

## Features

- **Hub-aware SSO** – Password and token login flows issue signed JWT sessions through Actix Identity so every Pushkind service can trust the same authenticated user.
- **Self-service onboarding** – Validated registration and profile forms let members join a hub, update their display name, and rotate credentials without operator help.
- **Role-based administration** – Administrators with `SERVICE_ACCESS_ROLE` can create hubs, manage roles, and assign permissions to users via guarded endpoints with flash feedback.
- **Configurable navigation menus** – Hub-specific menu links can be added or removed to surface curated destinations in the shared Tera layout.
- **Password recovery links** – ZeroMQ-backed email delivery sends short-lived recovery tokens that drop users back into the login flow safely.
- **JSON user directory** – `/api/v1` endpoints expose the hub’s user list with pagination, search, and role filters for downstream integrations.

## Architecture at a Glance

The codebase follows a clean, layered structure so that business logic can be
exercised and tested without going through the web framework:

- **Domain (`src/domain`)** – Type-safe models for hubs, menus, roles, and users.
  Domain types never validate or normalize; they assume inputs are already
  cleaned and transformed by forms/services.
- **Repository (`src/repository`)** – Traits that describe the persistence
  contract and a Diesel-backed implementation (`DieselRepository`) that speaks to
  a SQLite database. Each module translates between Diesel models and domain
  types and exposes strongly typed query builders.
- **Services (`src/services`)** – Application use-cases that orchestrate domain
  logic, repository traits, and Pushkind authentication helpers. Services return
  `ServiceResult<T>` and map infrastructure errors into well-defined service
  errors.
- **Forms (`src/forms`)** – `serde`/`validator` powered structs that handle
  request payload validation, CSV parsing, and transformation into domain types.
- **Routes (`src/routes`)** – Actix Web handlers that wire HTTP requests into the
  service layer and render Tera templates or redirect with flash messages.
- **Templates (`templates/`)** – Server-rendered UI built with Tera and
  Bootstrap 5, backed by sanitized HTML rendered via `ammonia` when necessary.

Because the repository traits live in `src/repository/mod.rs`, service functions
accept generic parameters that implement those traits. This makes unit tests easy
by swapping in the `mockall`-based fakes from `src/repository/mock.rs`.

## Technology Stack

- Rust 2024 edition
- [Actix Web](https://actix.rs/) with identity, session, and flash message
  middleware
- [Diesel](https://diesel.rs/) ORM with SQLite and connection pooling via r2d2
- [Tera](https://tera.netlify.app/) templates styled with Bootstrap 5.3
- [`pushkind-common`](https://github.com/pushkindt/pushkind-common) shared crate
  for authentication guards, configuration, database helpers, and reusable
  patterns
- Supporting crates: `chrono`, `validator`, `serde`, `ammonia`, `csv`, and
  `thiserror`

## Getting Started

### Prerequisites

- Rust toolchain (install via [rustup](https://www.rust-lang.org/tools/install))
- `diesel-cli` with SQLite support (`cargo install diesel_cli --no-default-features --features sqlite`)
- SQLite 3 installed on your system

### Environment

The service reads configuration from environment variables. The most important
ones are:

| Variable | Description | Default |
| --- | --- | --- |
| `DATABASE_URL` | Path to the SQLite database file | `app.db` |
| `SECRET_KEY` | 32-byte secret for signing cookies | generated at runtime |
| `PORT` | HTTP port | `8080` |
| `ADDRESS` | Interface to bind | `127.0.0.1` |
| `DOMAIN` | Cookie domain (without protocol) | `localhost` |

Create a `.env` file if you want these values loaded automatically via
[`dotenvy`](https://crates.io/crates/dotenvy).

### Database

Run the Diesel migrations before starting the server:

```bash
diesel setup
cargo install diesel_cli --no-default-features --features sqlite # only once
diesel migration run
```

A SQLite file will be created at the location given by `DATABASE_URL`.

## Running the Application

Start the HTTP server with:

```bash
cargo run
```

The server listens on `http://127.0.0.1:8080` by default and serves static
assets from `./assets` in addition to the Tera-powered HTML pages. Authentication
and authorization are enforced via the Pushkind auth service and the
`SERVICE_ACCESS_ROLE` constant.

## Quality Gates

The project treats formatting, linting, and tests as required gates before
opening a pull request. Use the following commands locally:

```bash
cargo fmt --all -- --check
cargo clippy --all-features --tests -- -Dwarnings
cargo test --all-features --verbose
cargo build --all-features --verbose
```

Alternatively, the `make check` target will format the codebase, run clippy, and
execute the test suite in one step.

## Testing

Unit tests exercise the service and form layers directly, while integration
tests live under `tests/`. Repository tests rely on Diesel’s query builders and
should avoid raw SQL strings whenever possible. Use the mock repository module to
isolate services from the database when writing new tests.

## Project Principles

- **Domain-driven**: keep business rules in the domain and service layers and
  translate to/from external representations at the boundaries.
- **Boundary sanitation**: perform validation and normalization (like email
  lowercasing) in forms/services so domain structs stay pure data.
- **Explicit errors**: use `thiserror` to define granular error types and convert
  them into `ServiceError`/`RepositoryError` variants instead of relying on
  `anyhow`.
- **No panics in production paths**: avoid `unwrap`/`expect` in request handlers,
  services, and repositories—propagate errors instead.
- **Security aware**: sanitize any user-supplied HTML using `ammonia`, validate
  inputs with `validator`, and always enforce role checks with
  `pushkind_common::routes::check_role`.
- **Testable**: accept traits rather than concrete types in services and prefer
  dependency injection so the mock repositories can be used in tests.

Following these guidelines will help new functionality slot seamlessly into the
existing architecture and keep the service reliable in production.
