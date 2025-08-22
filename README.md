# pushkind-auth

`pushkind-auth` is a single sign-on (SSO) authentication service that powers
the Pushkind ecosystem. Built with Rust, Actix Web and Diesel, it provides
session and token based authentication, user management and administrative
endpoints for other Pushkind services.

## Features

- Actix Web server with identity and session management
- SQLite database access via Diesel ORM
- REST API endpoints for user and role management
- Tera templates for server-rendered pages

## Running locally

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Set the required environment variables:
   - `DATABASE_URL` (e.g. `app.db`)
   - `SECRET_KEY` for session encryption
   - Optional: `PORT`, `ADDRESS`, `DOMAIN`
3. Run database migrations with `diesel migration run` (requires `diesel-cli`).
4. Start the server:

```bash
cargo run
```

The service listens on `http://127.0.0.1:8080` by default.

## Testing

Run the test suite with:

```bash
cargo test
```
