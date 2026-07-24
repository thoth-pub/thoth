# Repository and Environment Map

Status: Verified 2026-07-24  
Owner: CTO

This directory records the repository, branch, build, CI, release and deployment boundaries required for AI-led delivery.

## Files

- `branch-topology.md` - observed branch state, desired standard and normalization gates.
- `environments.md` - verified runtime, preview and release boundaries.
- `control-gaps.md` - missing controls and required follow-up tasks.
- `repositories/thoth.md`
- `repositories/thoth-app.md`
- `repositories/thoth-dissemination.md`
- `repositories/thoth-sphinx.md`
- `repositories/metrics-dashboard.md`
- `repositories/metrics-widget.md`
- `repositories/cc-license.md`

## Usage rule

At the start of every task:

1. read the relevant repository file;
2. verify the current branch and head commit directly;
3. verify commands against the current repository;
4. report any difference from this map;
5. update this map through a reviewed documentation task when durable repository behaviour changes.

This map is not a substitute for live GitHub state.
