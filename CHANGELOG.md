# Changelog
All notable changes to thoth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## Changed

## Added

## [0.1.2] - 2020-03-03
## Changed
- Port exposing is handled in Dockerfile instead of docker-compose
- Moved server start function from binary to library

## Added
- Added limit and offset arguments to all queries
- Added default order by clauses to all queries
- Implemented GraphQL errors for diesel errors
- Added filter arguments for publishers and works queries

## [0.1.1] - 2020-02-27
## Changed
- Improved Dockerfile to allow running database migrations at run time

## Added
- Implemented imprints for publisher graphql object
- Added subcommands to main binary to allow running embedded migrations without having to install diesel\_cli
- Automatic publication to crates.io

## [0.1.0] - 2020-02-21
## Added
- Database migrations
- GraphQL handlers implementing the thoth schema
