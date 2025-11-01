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

Initialise a new project:

```sh
surrealkit init
```

This creates a directory `/database` with the necessary scaffolding

The following ENV variables will be picked up for your `.env` file, SurrealKit assumes you're using SurrealDB as a Web Database.

- `PUBLIC_DATABASE_HOST`
- `PUBLIC_DATABASE_NAMESPACE`
- `PUBLIC_DATABASE_DATABASE`
- `DATABASE_USERNAME`
- `DATABASE_PASSWORD`

A table (`_migration`) is generated and managed by SurrealKit on your configured database.

### Seeding

Seeding will automatically run when you apply migrations. If you would like to reapply migrations, please re-apply your migrations.

```sh
surrealkit seed
```
