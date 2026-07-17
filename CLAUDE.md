# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`{{project-name}}` is a Cargo workspace scaffold - one or more gRPC backends
behind an axum HTTP gateway - generated from the
[rust-template-api](https://github.com/adrien-thebault/rust-template-api)
cargo-generate template. Nothing here is a monolith: each concern is its own
crate, and the gateway is the only HTTP-facing one.

This is
[rust-template-grpc](https://github.com/adrien-thebault/rust-template-grpc)'s
same `Todo` example (a familiar shape chosen to show the pattern without a
domain explanation getting in the way, not because a todo list is the actual
point), split into a workspace with a `web` gateway crate in front of it.
Rename/reshape `Todo` (and everything under `services/todo-service`,
`proto/src/todo-service.proto`) to fit the real domain, then add more
entities/services the same way - see the README's "Adding a second service"
section.

## Architecture

```
proto/                  - all .proto definitions + generated types, shared by every backend crate
services/todo-service/  - tonic+diesel/sqlite gRPC service (example CRUD entity: Todo)
web/                     - axum gateway, REST+JSON (:8080) - proxies to gRPC backends, owns auth/sessions
```

**Why a separate gateway instead of a service speaking HTTP directly**:
`services/todo-service` is pure gRPC (tonic). `web` is the only HTTP-facing
crate - it does auth (basic-auth login -> signed session token via
`rust_toolbox::axum_tools`), RBAC (`Role::Admin`-gated `/api/admin/*` routes
vs. public `/api/*` routes), and translates REST<->proto for every backend
behind it.

`TodoRepository` implements `rust_toolbox`'s `Repository`/`Find`/`Save`/
`Delete` traits via the `impl_repository!`/`impl_save!` macros rather than
hand-written CRUD; `TodoService` implements `DatabaseService` (pool -> tonic
server) and `EntityService<TodoRepository>` (find/save/delete request
handling) - look at `services/todo-service` before adding a new entity, the
pattern is meant to be copied, not reinvented.

## Commands

```sh
cargo build --workspace
cargo test --workspace
cargo test creates_a_new_todo  # single test (substring match)
cargo fmt --all && cargo clippy --workspace
```

`services/todo-service`'s integration tests spin up a real tonic server
against an in-memory (or `/tmp`-backed) sqlite db - no external DB setup
needed. Diesel migrations are embedded into the binary at compile time and
run automatically on startup; `diesel.toml` points `diesel print-schema` at
the right `schema.rs` if you add a migration by hand.

## Conventions to preserve

- **Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)**
  - see [CONTRIBUTING.md](CONTRIBUTING.md). A `commit-msg` hook enforcing
    this is opt-in (`git config core.hooksPath .githooks`), and
    `CHANGELOG.md` is generated from commit history with
    [git-cliff](https://git-cliff.org/) (`scripts/changelog.sh`) - a
    malformed type/scope means a commit silently drops out of, or gets
    miscategorized in, the changelog.
- Every service/entity follows the same repository/service/gRPC + gateway
  `Controller` shape as `todo-service`/`web/src/routes/todos.rs` - don't
  hand-write CRUD or introduce a different pattern for a new one.
- `web` is the only HTTP-facing crate - a new backend service stays pure gRPC
  and gets proxied through `web`, never exposes its own HTTP surface.
- Diesel `AsChangeset` skips `Option<T>` fields on `None` by default (doesn't
  null them out) - any nullable column that needs to be clearable needs
  `#[diesel(treat_none_as_null = true)]` on that field.
