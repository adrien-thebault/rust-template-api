# {{project-name}}

A Cargo workspace scaffold - one or more gRPC backends behind an axum HTTP
gateway - generated from
[rust-template-api](https://github.com/adrien-thebault/rust-template-api) via
[`cargo-generate`](https://github.com/cargo-generate/cargo-generate).

Nothing here is a monolith: each concern is its own crate, and the gateway is
the only HTTP-facing one. This is
[rust-template-grpc](https://github.com/adrien-thebault/rust-template-grpc)'s
same `Todo` example, split into a workspace with a `web` gateway crate in
front of it. If you don't need the gateway - just one bare gRPC service -
use rust-template-grpc instead.

## Architecture

```
proto/                     - all .proto definitions + generated types, shared by every backend crate
services/todo-service/     - tonic+diesel/sqlite gRPC service (example CRUD entity: Todo)
  src/
    model/todo.rs             - the diesel entity + its wire-conversion errors
    schema.rs                  - diesel::table! (regenerate with `diesel print-schema`)
    repository/todo_repository.rs - data access, via rust_toolbox::impl_repository!
    service/todo_service.rs    - the gRPC service impl (rust_toolbox::EntityService)
    proto.rs                   - conversions between crate::model and proto's generated types
  migrations/                - diesel migrations, embedded into the binary at compile time
  tests/                      - integration tests (repository + end-to-end gRPC)
web/                        - axum gateway, REST+JSON (:8080) - proxies to gRPC backends, owns auth/sessions
```

`web` is the only HTTP-facing crate: it does auth (basic-auth login ->
signed session token via `rust_toolbox::axum_tools`), RBAC (`Role::Admin`-
gated `/api/admin/*` routes vs. public `/api/*` routes), and translates
REST<->proto for every backend behind it. `services/todo-service` builds its
diesel repository via `rust_toolbox`'s `impl_repository!`/`impl_save!`
macros and implements `DatabaseService` + `EntityService<TodoRepository>`
rather than hand-writing CRUD - see
[rust-toolbox](https://github.com/adrien-thebault/rust-toolbox).

## Commands

```sh
cargo build --workspace
cargo test --workspace
cargo test creates_a_new_todo  # single test (substring match)
cargo fmt --all && cargo clippy --workspace
```

`services/todo-service`'s integration tests spin up a real tonic server
against an in-memory (or `/tmp`-backed) sqlite db - see
`services/todo-service/tests/common.rs` - no external DB setup needed.
Diesel migrations are embedded into the binary at compile time (`MIGRATIONS`
in `services/todo-service/src/lib.rs`) and run automatically on startup;
`diesel.toml` points `diesel print-schema` at the right `schema.rs` if you
add a migration by hand.

### Running the full stack locally

```sh
touch todo-service.db
docker-compose up --build
# -> http://localhost:8080/api/todos (public)
# admin routes at /api/admin/todos - see .env.example for the default
# admin/admin credentials (change WEB_ADMIN_PASSWORD_SHA256 for anything
# beyond local testing)
```

Or run each binary directly (`cargo run` from inside
`services/todo-service`, `web`) - each picks up `.env` via `dotenvy` and
has sane localhost defaults for the other's address (`TODO_SERVICE_ADDR`).

Get an admin session token, then call an admin route:

```sh
TOKEN=$(curl -s -X POST localhost:8080/api/auth/login \
  -H 'content-type: application/json' \
  -d '{"username":"admin","password":"admin"}' | jq -r .token)

curl -X POST localhost:8080/api/admin/todos \
  -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"title":"first"}'
```

## Adding a second service

1. Add a new proto file under `proto/src/*.proto` (`proto`'s `build.rs`
   globs every `proto/src/**/*.proto`) and a `pub mod your_service { tonic::
   include_proto!(...) }` block in `proto/src/lib.rs`.
2. Copy `services/todo-service` as `services/your-service` (model, schema,
   migrations, repository, service, tests) and add it to the root
   `Cargo.toml`'s `[workspace] members`.
3. Wire a client for it into `web`'s `AppState` (`web/src/main.rs`) and add a
   `web/src/routes/your_resource.rs` `Controller`, merged into
   `web/src/routes.rs`'s `router()`.
4. Add a `runtime-your-service` stage to the `Dockerfile` and a matching
   service block to `docker-compose.yml`, following the `todo-service` ones.

## Swapping the database backend

This template hardcodes SQLite (`rust-toolbox`'s `sqlite` feature, diesel's
`sqlite` feature, `returning_clauses_for_sqlite_3_35`, both set via the root
`Cargo.toml`'s `[workspace.dependencies]`). To use PostgreSQL or MySQL
instead: swap those two feature flags for `postgresql`/`mysql`
(`rust-toolbox`'s `impl_save!` already has a MySQL-specific arm, since MySQL
has no `RETURNING`), and adjust every `migrations/*/up.sql`'s
`AUTOINCREMENT`/timestamp syntax accordingly.
