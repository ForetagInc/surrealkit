# Testing Examples (Root + Access User)

This folder provides a full example for SurrealKit's `surrealkit test` runner that covers:

- Tables and field type checks (`string`, `number`, `datetime`)
- Computed field assertions
- Function assertions
- Graph/edge assertions (`RELATE` with relation table)
- Root user vs record access-user permissions
- Record access account provisioning through `SIGNUP`, followed by authentication through `SIGNIN`
- API endpoint examples for both root and access user

## Files

- `config.toml`: global test config with `root` and `access_user` actors
- `fixtures/root_and_access_setup.surql`: schema/access/function/api setup
- `suites/root_access_full_stack.toml`: end-to-end suite

## Reports

Copy these into your active test directories:

```sh
cp /examples/testing/config.toml /database/tests/config.toml
cp /examples/testing/suites/root_access_full_stack.toml /database/tests/suites/root_access_full_stack.toml
cp /examples/testing/fixtures/root_and_access_setup.surql /database/tests/fixtures/root_and_access_setup.surql
```

Then run:

```sh
surrealkit test --suite '*root_access_full_stack*' --json-out database/tests/report.json
```

## Notes

- The suite uses a `record` access actor (`kind = "record"`) with `access = "app_access"`.
- `access_user` is created by the runner with `signup_params`, then authenticated with `signin_params`, so the test setup exercises the access method instead of relying on a pre-created fixture account.
- API paths (`/api/v1/health`, `/api/v1/profile`) are placeholders. Update them to match your API routes.
- The runner uses ephemeral namespace/database isolation by default.
