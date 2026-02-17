# SurrealKit

[![Crates.io](https://img.shields.io/crates/v/surrealkit.svg)](https://crates.io/crates/surrealkit) [![Documentation](https://docs.rs/surrealkit/badge.svg)](https://docs.rs/surrealkit)
[![License](https://img.shields.io/badge/license-Unlicense-blue.svg)](https://unlicense.org/)

> NOT FOR PRODUCTION USE | For SurrealDB v3

Manage SurrealDB migrations, seeding, and testing with ease. Inspired by Eloquent ORM's migration pattern.

## Scope

This project is designed to manage SurrealDB migrations, seed, testing, and database management. It is not intended for production use and is specifically tailored for SurrealDB version 3.

If and when SurrealDB implements first-class tooling to manage migrations, seeding, and testing, SurrealKit will be deprecated in favour of the official SurrealDB tooling but intends to provide seamless transition.

## Usage

Install via Cargo:

```sh
cargo install surrealkit
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/ForetagInc/surrealkit/releases) (Linux, macOS Intel/Apple Silicon, Windows).

Initialise a new project:

```sh
surrealkit init
```

This creates a directory `/database` with the necessary scaffolding

The following ENV variables will be picked up for your `.env` file, SurrealKit assumes you're using SurrealDB as a Web Database.

- `PUBLIC_DATABASE_HOST`
- `PUBLIC_DATABASE_NAME`
- `PUBLIC_DATABASE_NAMESPACE`
- `DATABASE_USERNAME`
- `DATABASE_PASSWORD`

A table (`_migration`) is generated and managed by SurrealKit on your configured database.

## Team Workflow

SurrealKit now separates schema authoring, dev sync, and deploy migrations:

1. Edit desired state in `database/schema/*.surql`
2. Reconcile dev DB with auto-prune:

```sh
surrealkit sync
```

3. Watch mode for local development:

```sh
surrealkit sync --watch
```

4. Generate a git-reviewed migration diff and update snapshots:

```sh
surrealkit commit --name add_customer_indexes
```

Generated migrations are written to `database/migrations/*.surql`.
Snapshots are tracked in:

- `database/.surrealkit/schema_snapshot.json`
- `database/.surrealkit/catalog_snapshot.json`

To guard CI against missing migration/snapshot updates:

```sh
surrealkit commit --dry-run
```

If prune is enabled against a shared DB, SurrealKit requires explicit override:

```sh
surrealkit sync --allow-shared-prune
```

### Seeding

Seeding will automatically run when you apply migrations. If you would like to reapply migrations, please re-apply your migrations.

```sh
surrealkit seed
```

## Testing Framework

```sh
surrealkit test
```

The runner executes declarative TOML suites from `database/tests/suites/*.toml` and supports:

- SQL assertion tests (`sql_expect`)
- Permission rule matrices (`permissions_matrix`)
- Schema metadata assertions (`schema_metadata`)
- Schema behavior assertions (`schema_behavior`)
- HTTP API endpoint assertions (`api_request`)

By default, each suite runs in an isolated ephemeral namespace/database and fails CI on any test failure.

### CLI Flags

`surrealkit test` supports:

- `--suite <glob>`
- `--case <glob>`
- `--tag <tag>` (repeatable)
- `--fail-fast`
- `--parallel <N>`
- `--json-out <path>`
- `--no-setup`
- `--no-sync`
- `--no-seed`
- `--base-url <url>`
- `--timeout-ms <ms>`
- `--keep-db`

### Global Config

Global test settings live in `database/tests/config.toml`.

Example:

```toml
[defaults]
timeout_ms = 10000
base_url = "http://localhost:8000"

[actors.root]
kind = "root"
```

Optional env fallbacks:

- `SURREALKIT_TEST_BASE_URL`
- `SURREALKIT_TEST_TIMEOUT_MS`
- `PUBLIC_DATABASE_HOST` (used as API base URL fallback when test-specific base URL is not set)

### Example Suite

```toml
name = "security_smoke"
tags = ["smoke", "security"]

[[cases]]
name = "guest_cannot_create_order"
kind = "sql_expect"
actor = "guest"
sql = "CREATE order CONTENT { total: 10 };"
allow = false
error_contains = "permission"

[[cases]]
name = "orders_api_returns_200"
kind = "api_request"
actor = "root"
method = "GET"
path = "/api/orders"
expected_status = 200

[[cases.body_assertions]]
path = "0.id"
exists = true
```

### Actor Example (Namespace / Database / Record / Token / Headers)

```toml
[actors.reader]
kind = "database"
namespace = "app"
database = "main"
username_env = "TEST_DB_READER_USER"
password_env = "TEST_DB_READER_PASS"

[actors.jwt_actor]
kind = "token"
token_env = "TEST_API_JWT"

[actors.custom_client]
kind = "headers"
headers = { "x-tenant-id" = "tenant_a" }
```

### Permission Matrix Example

```toml
[[cases]]
name = "reader_permissions"
kind = "permissions_matrix"
actor = "reader"
table = "order"
record_id = "perm_test"

[[cases.rules]]
action = "select"
allow = true

[[cases.rules]]
action = "update"
allow = false
error_contains = "permission"
```

### JSON Reports for CI

Generate machine-readable output:

```sh
surrealkit test --json-out database/tests/report.json
```

The command exits non-zero if any case fails.
