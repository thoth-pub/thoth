# Repository: thoth-pub/metrics-widget

## Responsibility

Embeddable React/JavaScript metrics package published to npm.

## Branches

GitHub default/release: `main`  
Development: `dev`  
CI branch filters: `main`, `dev`  
Target: `develop -> master`

BR-WIDGET-01 must normalize branch names and preserve release automation.

## Stack

- React 19 peer dependency
- TypeScript 5.9
- Vite 7
- Biome
- API Extractor
- generated declaration/package output in `dist/`

## Commands

```bash
npm ci
npm run lint
npm run build
npm run test:consumer
npm pack --dry-run
```

## CI

Current CI runs:

- lint;
- build;
- consumer smoke test.

No unit-test suite was detected.

## Release

A published GitHub release:

1. checks out the release;
2. verifies the tag is `v<package.json version>`;
3. lints;
4. builds;
5. checks package contents;
6. runs consumer smoke;
7. publishes to npm.

## Metrics migration invariants

- Preserve React and vanilla-JavaScript consumers.
- Preserve public package exports and CSS isolation.
- Do not embed machine credentials.
- The future authenticated path must be compatible with third-party embedding.
- Add tests for data failure, coverage and partial response semantics before cutover.
- A release is high risk because it publishes a public package.
