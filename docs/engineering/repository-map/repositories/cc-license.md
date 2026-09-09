# Repository: thoth-pub/cc-license

Evidence date: repository control and the Thoth crate dependency verified
2026-08-16

## Responsibility

Canonical Rust parser and metadata authority for Creative Commons licence URLs retained by Thoth.

## Branches

GitHub default/release: `main`
Development: `develop`
Established `<release-branch>`: `main`, **preserved**

Under [`ADR-0011`](../../decisions/ADR-0011-preserve-established-release-branch-names.md) `main` is this repository's established release/default branch and is preserved. No `master` branch is created and no `main` to `master` conversion is required for naming consistency.

Verified `develop` head: `3dd497981da5d540739158d086394d22b3146b25`
(2026-08-16).

BR-LIC-01 must complete the remaining publication readiness — CI filters and
protections for the verified branches, and the crate publication, approval and
rollback/yank procedure recorded under "Release gap" below — before
publication. It remains a separate, separately authorized and unimplemented
task.

## Repository control

Repository-local root `AGENTS.md` merged onto `develop` through PR
[#2](https://github.com/thoth-pub/cc-license/pull/2), verified live 2026-08-16
at `3dd497981da5d540739158d086394d22b3146b25`. Later work must read and preserve
it rather than add one as though absent.

That control PR was blocked by a pre-existing Clippy `ToString` gate failure,
which supporting repair PR
[#4](https://github.com/thoth-pub/cc-license/pull/4) (issue
[#3](https://github.com/thoth-pub/cc-license/issues/3)) fixed by implementing
`Display` for `License`; it merged first, as
`2d9b0c798c23a14131edeeb7fd525188500882dd`. **No crate publication occurred**
as part of that work, and none is authorized by this record.

## Stack

- Rust 2021
- crate name `cc_license`
- regex-based parsing
- crates.io distribution documented

## Commands

```bash
cargo test --workspace --verbose
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## CI

Current CI runs tests, clippy and formatting on `main` and `develop`.

The workflow uses older checkout/actions-rs actions. Modernization should be a separate CI task from licence behaviour.

## Publisher Services requirements

LIC-01 will add or confirm:

- canonical URL output;
- enumeration of retained licences;
- descriptions and display metadata;
- legacy/jurisdiction metadata;
- public-domain tools retained by Thoth;
- rejection of invalid/spoofed URLs;
- normalization tests.

Thoth consumes a released crate version rather than copying parsing rules, and
already does so: `thoth-export-server` depends on `cc_license = "0.1.0"`
(verified 2026-08-16 in `thoth-export-server/Cargo.toml`, resolved in
`Cargo.lock` to `0.1.0` from crates.io). That published release predates the
current engineering-control programme.

A breaking change to the crate's public API, or a new release Thoth is expected
to adopt, must be assessed against `thoth-export-server` and reaches Thoth only
through a deliberate, separately authorized dependency bump. See
[`contracts.md`](../contracts.md) section 2.5.

## Release gap

The exact crate publication command, credentials, approval and rollback/yank procedure are not verified. Record them before LIC-01 is marked production ready.
