# Changelog
All notable changes to thoth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## \[Unreleased\]
### Changed
  - Pin compiler's docker image to a specific version (best practice)
  - Use COPY instead of ADD for directories in Dockerfile (best practice)

### Added
  - [#23](https://github.com/openbookpublishers/thoth/issues/23) - Redirect "/" to "/graphiql"
  - [#18](https://github.com/openbookpublishers/thoth/issues/18) - Create ThothError structure to start catching all other types of errors

## [[0.1.2]](https://github.com/OpenBookPublishers/thoth/releases/tag/v0.1.2) - 2020-03-03
### Changed
  - [#10](https://github.com/openbookpublishers/thoth/issues/10) - Port exposing is handled in Dockerfile instead of docker-compose
  - [#16](https://github.com/openbookpublishers/thoth/issues/16) - Moved server start function from binary to library
  - [#9](https://github.com/openbookpublishers/thoth/issues/9) - Docker image is now compiled statically

### Added
  - [#13](https://github.com/openbookpublishers/thoth/issues/13) - Added limit and offset arguments to all queries
  - [#13](https://github.com/openbookpublishers/thoth/issues/13) - Added default order by clauses to all queries
  - [#15](https://github.com/openbookpublishers/thoth/issues/15) - Implemented GraphQL errors for diesel errors
  - [#13](https://github.com/openbookpublishers/thoth/issues/13) - Added filter arguments for publishers and works queries

## [[0.1.1]](https://github.com/OpenBookPublishers/thoth/releases/tag/v0.1.1) - 2020-02-27
### Changed
  - Improved Dockerfile to allow running database migrations at run time

### Added
  - Implemented imprints for publisher graphql object
  - [#6](https://github.com/openbookpublishers/thoth/issues/6) - Added subcommands to main binary to allow running embedded migrations without having to install diesel\_cli
  - Automatic publication to crates.io

## [[0.1.0]](https://github.com/OpenBookPublishers/thoth/releases/tag/v0.1.0) - 2020-02-21
### Added
  - Database migrations
  - GraphQL handlers implementing the thoth schema
