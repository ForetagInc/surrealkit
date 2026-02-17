# Testing Examples (Root + Access User)

This folder provides a full example for SurrealKit's `surrealkit test` runner that covers:

- Tables and field type checks (`string`, `number`, `datetime`)
- Computed field assertions
- Function assertions
- Graph/edge assertions (`RELATE` with relation table)
- Root user vs record access-user permissions
- API endpoint examples for both root and access user

## Files

- `config.toml`: global test config with `root` and `access_user` actors
- `fixtures/root_and_access_setup.surql`: schema/access/function/api setup
- `suites/root_access_full_stack.toml`: end-to-end suite

## Use In This Repo

Copy these into your active test directories:

```sh
cp /Users/chiru/Projects/OSS/surrealkit/examples/testing/config.toml /Users/chiru/Projects/OSS/surrealkit/database/tests/config.toml
cp /Users/chiru/Projects/OSS/surrealkit/examples/testing/suites/root_access_full_stack.toml /Users/chiru/Projects/OSS/surrealkit/database/tests/suites/root_access_full_stack.toml
cp /Users/chiru/Projects/OSS/surrealkit/examples/testing/fixtures/root_and_access_setup.surql /Users/chiru/Projects/OSS/surrealkit/database/tests/fixtures/root_and_access_setup.surql
```

Then run:

```sh
surrealkit test --suite '*root_access_full_stack*' --json-out database/tests/report.json
```

## Notes

- The suite uses a `record` access actor (`kind = "record"`) with `access = "app_access"`.
- API paths (`/api/v1/health`, `/api/v1/profile`) are placeholders. Update them to match your API routes.
- The runner uses ephemeral namespace/database isolation by default.
