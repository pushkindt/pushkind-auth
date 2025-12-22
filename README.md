# pushkind-auth

Implementation must conform to SPEC.md

`pushkind-auth` is the Pushkind single sign-on (SSO) service. See `SPEC.md` for
architecture, routes, and operational details.

## Getting Started

### Prerequisites

- Rust toolchain (install via [rustup](https://www.rust-lang.org/tools/install))
- `diesel-cli` with SQLite support (`cargo install diesel_cli --no-default-features --features sqlite`)
- SQLite 3 installed on your system

### Database

Run the Diesel migrations before starting the server:

```bash
diesel setup
cargo install diesel_cli --no-default-features --features sqlite # only once
diesel migration run
```

A SQLite file will be created at the location given by `APP_DATABASE_URL`.

## Running the Application

Start the HTTP server with:

```bash
cargo run
```

The server listens on `http://127.0.0.1:8081` by default (from
`config/local.yaml`) and serves static
assets from `./assets` in addition to the Tera-powered HTML pages. Authentication
and authorization are enforced via the Pushkind auth service and the
`SERVICE_ACCESS_ROLE` constant.
