# Changelog
All notable changes to thoth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## Changed
- Improved Dockerfile to allow running database migrations at run time

## Added
- Implemented imprints for publisher graphql object
- Created a binary to run embedded migrations to avoid installing diesel\_cli

## [0.1.0] - 2020-02-21
## Added
- Database migrations
- GraphQL handlers implementing the thoth schema
