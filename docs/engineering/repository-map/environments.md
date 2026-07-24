# Environment and Deployment Map

Status: Partially verified
Evidence date: 2026-07-24

## 1. Verified environments

### Thoth App

Hosting: Vercel
Framework: Next.js
Node: 22.x
Production branch observed: `main`
Preview branch observed: `dev`
Production domain: `admin.thoth.pub`

Production deployments are created from merges into `main`. Feature/development commits receive Vercel preview deployments.

### Metrics Dashboard

Hosting: Vercel
Framework: Next.js
Node: 22.x
Production branch observed: `main`
Development branch: `develop`
Production domain: `metrics.thoth.pub`

### Metrics Widget

Delivery: npm package
Release trigger: published GitHub release
Validation before publish:

- release tag equals `v<package.json version>`;
- lint;
- build;
- package dry run;
- consumer smoke test.

### Thoth

Release artefact: container image in GHCR
Trigger: published GitHub release
Image: `ghcr.io/thoth-pub/thoth`

The production compute platform, database migration execution path, deployment approval and rollback procedure were not verified in CTRL-02 and require a follow-up operations inventory.

### Thoth Dissemination

Release artefact: Docker Hub image
Trigger: published GitHub release
Image: `openbookpublishers/thoth-dissemination`

Operational execution also occurs through GitHub Actions. Some workflows can write to Thoth and external platforms and use protected credentials/environments.

### Thoth Sphinx

Current environment: none verified
Planned architecture:

- scheduled ECS/Fargate tasks;
- EventBridge;
- private S3;
- AWS Secrets Manager or SSM;
- CloudWatch;
- manual GitHub Actions with AWS OIDC.

Planned architecture must not be documented as deployed state.

### cc-license

Delivery: Rust crate
Registry: crates.io is documented in the repository README.
The exact publication procedure and release owner were not verified.

## 2. API endpoints referenced by repositories

- GraphQL production: `https://api.thoth.pub/graphql`
- Export production: `https://export.thoth.pub`
- GraphQL test/codegen: `https://api.test.thoth.pub/graphql`
- Current OPERAS metrics API: `https://metrics-api.operas-eu.org`

Do not hardcode these into new domain logic when environment configuration is appropriate.

## 3. Environment controls still required

- branch-to-environment map after branch normalization;
- production deployment owners;
- Thoth API/export runtime platform;
- database migration execution and approval;
- rollback and restore commands;
- feature-flag ownership;
- service-role and secret-rotation ownership;
- staging equivalents for Sphinx and metrics ingestion;
- observability dashboard locations.

No production task may infer these missing controls.
