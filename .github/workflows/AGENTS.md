# AGENTS.md - GitHub Actions workflows

This file extends the repository-root `AGENTS.md`.

Workflow changes can affect CI, releases, external services and production data. Classify the risk from actual effects, not from the size of the YAML diff.

## 1. Prohibited actions

An implementing agent must not:

- dispatch a production or write-capable workflow;
- approve a protected environment;
- retrieve or print production secrets;
- publish a release or container;
- change production deployment behaviour without explicit authorization;
- remove safety confirmations, environment protection or concurrency controls to make a test pass.

Use static validation, unit tests, forks, mocks or explicitly approved non-production environments.

## 2. Permissions and credentials

Use least-privilege `permissions`.

Prefer:

- GitHub OIDC and short-lived credentials;
- protected environments for write paths;
- environment-scoped secrets;
- read-only jobs for discovery and dry runs;
- separate jobs for selection and mutation.

Never echo secrets, tokens or complete credential-bearing URLs.

A read-only/dry-run mode should not receive write credentials.

## 3. Branch and trigger safety

Use the repository's actual branch topology:

```text
develop -> master
```

Before changing filters, verify:

- PR checks still run on feature and programme branches;
- release jobs run only from the approved release mechanism;
- manual write jobs enforce the intended source branch;
- scheduled jobs cannot accidentally run from an unreviewed branch.

Do not use branch normalization as an incidental part of a workflow feature task.

## 4. Durable state and concurrency

GitHub Actions is not a durable job ledger, lock manager or source of truth.

For operational workflows:

- keep durable jobs/checkpoints in Thoth;
- use explicit concurrency groups to prevent unsafe overlap;
- set `cancel-in-progress` deliberately;
- bound retries and matrices;
- ensure repeat execution is idempotent;
- report overflow rather than silently dropping work;
- keep matrix sizes within GitHub limits;
- preserve complete diagnostic artifacts where safe.

## 5. Manual mutation controls

A write-capable manual workflow should normally require:

- an explicit typed confirmation value;
- a protected environment;
- bounded scope;
- a reviewed dry run;
- fail-closed configuration;
- sanitized logs;
- a final summary;
- a documented rollback or reconciliation path.

An empty publisher/work/platform selection must never broaden to all records.

## 6. Validation

For YAML-only changes, run the repository's established actionlint procedure.

Also run any unit tests that parse or assert workflow behaviour.

At minimum:

```bash
git diff --check
```

Inspect:

- triggers;
- permissions;
- environments;
- secrets/variables;
- conditionals;
- concurrency;
- matrix size;
- failure propagation;
- artifact retention;
- branch guards.

CI success does not prove that a production write path is safe.
