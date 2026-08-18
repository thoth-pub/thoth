# Repository: thoth-pub/baboon

Evidence date: 2026-08-16

## Responsibility

Library-oriented MARC exchange orchestration for Thoth: a Rust package
(`baboon` library plus `baboon` binary) that generates and delivers per-library
MARC batches from Thoth metadata.

It owns:

- library MARC exchange orchestration — the feed run order and failure
  semantics (`src/run.rs`);
- Thoth metadata and export consumption — the GraphQL discovery client
  (`src/thoth_graphql.rs`) and the MARC export client (`src/marc_export.rs`);
- classification and batch/manifest generation — new/updated/deleted
  classification against per-library state, `.mrc` batch files and the JSON
  manifest (`src/classifier.rs`, `src/batches.rs`, `src/folders.rs`);
- persistent state and MARC cache orchestration — per-library state and the
  exported-MARC cache, and their sync-down/sync-up against S3-compatible object
  storage (`src/state.rs`, `src/marc_cache.rs`, `src/object_store.rs`);
- SFTP delivery — upload of batch files and manifests to each library's SFTPGo
  drop, including strict host-key verification and the put/rename/verify
  pattern (`src/sftp.rs`, `src/atomic_io.rs`);
- the library configuration schema and pilot provisioning tooling
  (`src/config.rs`, `config/libraries.example.yml`,
  `scripts/provision_sftpgo_pilot.py`).

The live library configuration (`config/libraries.yml`) is deliberately **not
committed**; it lives in object storage and is `.gitignore`d.

## Visibility

Private.

## Branches

GitHub default/release: `master`
Active integration: `develop`
Observed release flow: `develop -> release/* -> master`, tagged (for example
`v0.5.0`) and merged back into `develop`
Target-policy state: conforms to the `develop -> master` pattern in
[`branch-topology.md`](../branch-topology.md)

Verified branch evidence:

- `develop`: `bdf0ee33b6e93179ac76b4ad514a6e71627825d3`;
- `master`: `36f83a176fc4b195a3ff24c75302c1f2dbf53b1c`;
- `compare/master...develop` reports `ahead_by: 3, behind_by: 0`.

Implementation work branches from the verified `develop` head. Verify the
actual base commit before branching; do not target normal implementation
directly at `master`.

## Stack

- Rust 2021
- `reqwest` (rustls TLS) for Thoth GraphQL and export HTTP
- `ssh2` for SFTP delivery
- `aws-config` / `aws-sdk-s3` with `tokio` for S3-compatible object storage
- `serde` / `serde_json` / `serde_yaml`, `clap`, `chrono`, `uuid`, `sha2`
- Python 3.12 for the one-shot SFTPGo provisioning script and its tests

## Repository control

Repository-local root `AGENTS.md` merged onto `develop` through PR
[#16](https://github.com/thoth-pub/baboon/pull/16), verified live 2026-08-16 at
`bdf0ee33b6e93179ac76b4ad514a6e71627825d3`. It is the authoritative
repository-local specialization of the Shared Engineering Controls and must be
read and preserved by later work rather than recreated as though absent.

## Contract relationships

Consumes (verified from `.github/workflows/library-marc-feeds.yml` and the
clients named above):

- the production Thoth GraphQL API, `https://api.thoth.pub/graphql`
  (`THOTH_GRAPHQL_URL`), owned by `thoth-pub/thoth`;
- the production Thoth metadata export API, `https://export.thoth.pub`
  (`THOTH_EXPORT_BASE_URL`), owned by `thoth-pub/thoth`.

Baboon is a downstream consumer of both and owns neither. A breaking or
semantically significant upstream change — schema, nullability, enum values,
authorization semantics, pagination, export formats, or export availability for
withdrawn or unsubscribed works — requires an explicit impact analysis against
Baboon recorded on the upstream task. Baboon must not guess an unmerged
upstream contract: wait for the upstream change to merge, or consume an
explicitly pinned preview named in the task specification.

See [`contracts.md`](../contracts.md) section 2.1.

## External state and delivery

- **Object storage.** S3-compatible object storage holds per-library state, the
  exported-MARC cache and the live library configuration, addressed by
  `BABOON_OBJECT_BUCKET`, `BABOON_OBJECT_PREFIX` and
  `BABOON_OBJECT_ENDPOINT_URL`.
- **Delivery.** Batch files and manifests are delivered to libraries through
  SFTPGo.

The bucket, prefix, credentials, lifecycle policy, and the SFTPGo service,
users, folders and permissions are operational infrastructure that this
repository does not own.

## Workflow classes

Three workflow classes exist and must not be conflated.

### A. Ordinary CI - `.github/workflows/ci.yml`

Triggers: `push`, `pull_request`. Jobs: `tests` (`cargo test`), `lint`
(`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
warnings`), `audit` (`cargo audit`) and `provisioning-script-tests`
(`pytest tests/test_provision_sftpgo.py`). Permissions: `contents: read`. Local
quality checks only, with no intended production external write.

### B. PR live SFTP tests - `.github/workflows/live-tests.yml`

Triggers: `pull_request`, `workflow_dispatch`. **Risk: HIGH.**

For same-repository (branch) pull requests, where repository secrets are
available, this workflow reads the `BABOON_SFTP_*` secrets, sets
`BABOON_LIVE_WRITE_TEST=1`, and runs
`cargo test --test live_sftp -- --ignored --nocapture --test-threads=1`
against the **production** SFTPGo service. Beyond read-only connectivity and
host-key verification, that executes the real
put → rename → verify → delete → verify-cleanup sequence, creating the scratch
directory if absent, confined to the unwatched scratch folder

```text
/manchester/.ci-healthcheck/
```

which no library import profile targets. It must never target a delivered feed
folder. Fork and Dependabot pull requests receive empty secrets and skip.

**Consequence: creating or updating a pull request in this repository has an
automatic, material external side effect on a production service.** Pull-request
authorization is therefore a distinct permission that must be granted
explicitly, with that side effect named, and the effect that actually occurred
must be observed and reported. Never disable, bypass, edit, rename or
condition-guard this workflow, or convert a branch pull request into a fork
pull request, to evade the gate.

### C. Production MARC feed - `.github/workflows/library-marc-feeds.yml`

Trigger: `workflow_dispatch` only; the daily `schedule:` cron is commented out
during the pilot. It consumes production Thoth GraphQL and export, reads and
writes object-storage state, cache and library configuration, and delivers
batches and manifests to SFTPGo.

Manual dispatch is a **production operation** requiring separate, explicit
authorization naming the intended effect, including `sample_size` and
`sample_seed` for a pilot deposit, which is refused by design if the library
already has export state.

## Feed safety invariant

> Remote state and cache sync-up happens **only after every SFTP upload has
> succeeded.**

If `run-local` succeeds but any upload fails, remote state stays at its previous
version, so the next run syncs that state down and regenerates the failed batch.
Successful new/updated exports cache the exact exported MARC bytes, because
deleted-title batches are generated from the cached last-exported record and
cannot rely on Thoth still being able to export a withdrawn or unsubscribed
work.

Any change affecting step ordering, failure propagation, partial-upload
handling, state-write timing, cache retention or classification inputs requires
heightened review.

## Non-responsibilities

Baboon does **not** own:

- the Thoth GraphQL and export contracts, which belong to `thoth-pub/thoth`;
- SFTPGo infrastructure — the service, its users, folders, permissions and
  credentials. This repository holds provisioning tooling and runbooks;
  possessing the tooling is not authorization to run it against the service;
- object-storage infrastructure — the bucket, prefix, credentials and lifecycle
  policy;
- library-side ingestion of a delivered batch.

## Prohibited assumptions

- Do not assume this repository shares `thoth`'s branch names beyond `develop`;
  its default and release branch is `master`.
- Do not treat the existence of a workflow, runbook or provisioning script as
  authorization to run it.
- Do not open or update a pull request here without authorization that names
  the automatic production SFTP scratch write.

This record documents verified repository state. It does not itself authorize
any production operation, workflow dispatch, secret access, external write,
release, deployment or activation.
