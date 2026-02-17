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
