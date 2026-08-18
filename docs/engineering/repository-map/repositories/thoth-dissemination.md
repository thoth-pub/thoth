# Repository: thoth-pub/thoth-dissemination

## Responsibility

Existing execution engine for delivering metadata and files to external distribution and preservation platforms.

## Branches

GitHub default/release: `main`
Development: `develop`
Normal feature PRs currently target `develop`
Target release branch: `master`

BR-DIS-01 must normalize the release branch before final programme release.

## Stack

- Python 3.11
- platform-specific uploader modules
- Docker
- GitHub Actions
- external APIs, SFTP/SWORD and object storage
- Thoth API/location write-back

## Mandatory orientation

Before editing:

- `AGENTS.md`
- `README.md`
- `disseminator.py`
- `obtain_new_ids.py`
- relevant `*uploader.py`
- location write-back scripts
- relevant `requirements*.txt`
- relevant workflow YAML
- tests for the affected path
- applicable task specification

## Commands

```bash
python -m pip install -r requirements.txt
python -m unittest discover -v
python -m compileall -q .
```

When workflows change, validate YAML/action syntax with the repository's established actionlint procedure.

## CI

GitHub CI runs:

- Python 3.11;
- full unittest discovery;
- source compilation.

## Release and operations

Published GitHub releases build and publish:

```text
openbookpublishers/thoth-dissemination
```

GitHub Actions also perform scheduled/manual operational work. Some paths can:

- upload to external platforms;
- modify external metadata;
- write locations to Thoth;
- process publisher catalogues.

Dry run and read-only discovery must remain credential-free where designed.

## Contract relationships

Verified consumer of the Thoth API for location write-back and publisher/work
discovery, owned by `thoth-pub/thoth`. See
`docs/engineering/repository-map/contracts.md` section 2.1. A breaking API
change affecting location write-back is a cross-repository impact that must
be assessed before scope is approved.

## Programme effects

Publisher Services:

- API-owned publisher discovery;
- env/compare/api modes;
- durable back-catalogue job worker;
- linked OAPEN/DOAB adapter deduplication.

Metrics:

- no canonical metrics responsibility after Sphinx exists.

## Prohibited operations

Implementing agents must not:

- dispatch production workflows;
- use production credentials;
- submit real works;
- run apply modes;
- remove configured publisher lists before comparison/cutover approval;
- treat an empty API assignment as permission to process all works.
