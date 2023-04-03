# Changelog
All notable changes to thoth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
  - [42](https://github.com/thoth-pub/thoth/pull/42) - Generate MARC 21 records
  - New `work` field `bibliography_note`

## [[0.9.18]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.18) - 2023-03-27
### Security
  - Upgrade `r2d2` to v0.8.10
  - Upgrade `scheduled-thread-pool` to v0.2.7
  - Upgrade `openssl` to v0.10.48
  - Upgrade `remove_dir_all` to v0.5.3

## [[0.9.17]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.17) - 2023-03-25
### Changed
  - Upgrade rust to `1.68.1` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.6.2`, node `v18.15.0` and rollup `v3.20.2`) in production and development `Dockerfile`
  - Upgrade `wasm-pack` to v0.11.0

## [[0.9.16]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.16) - 2023-03-24
### Added
  - [#480](https://github.com/thoth-pub/thoth/pull/480) Add field to work table to track when the work or any of its relations was last updated

### Changed
  - Removed manual character checks and derivable defaults to comply with [`rustc 1.68.0`](https://github.com/rust-lang/rust/releases/tag/1.68.0)
  - [484](https://github.com/thoth-pub/thoth/pull/484) GraphQL queries: support filtering on multiple enum variants for work status and language relation, and add filtering for works last updated before/after a specified timestamp

## [[0.9.15]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.15) - 2023-03-01
### Fixed
  - Issue adding institutions in previous release

## [[0.9.14]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.14) - 2023-03-01
### Changed
  - Upgrade `openssl-src` to v111.25.0
  - Upgrade `bumpalo` to v3.12.0

### Fixed
  - [#326](https://github.com/thoth-pub/thoth/issues/326) - Debounce search queries

## [[0.9.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.13) - 2023-02-21
### Changed
  - Input actix keep alive via CLI arguments
  - Implement a failed request retry policy in client

## [[0.9.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.12) - 2023-02-17
### Changed
  - Reduce number of concurrent requests

## [[0.9.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.11) - 2023-02-17
### Changed
  - Upgrade rust to `1.67.1` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.5.0`, node `v18.14.1` and rollup `v3.15.0`) in production and development `Dockerfile`

## [[0.9.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.10) - 2023-02-17
### Changed
  - Include `limit` and `offset` in `thoth-client`'s works query
  - Paginate `get_works` requests in export API using concurrent requests
  - Input number of actix workers via CLI arguments

### Added
  - Work count query to `thoth-client`

## [[0.9.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.9) - 2023-02-16
### Changed
  - Upgrade `actix-web` to v4.3.0
  - Upgrade `actix-cors` to v0.6.4
  - Upgrade `env_logger` to v0.10.0
  - Upgrade `jsonwebtoken` to v8.2.0
  - Upgrade `strum` to v0.24.1
  - Output real IP address in actix logs

## [[0.9.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.8) - 2023-02-14
### Changed
  - Replace generic error with actual message when migrations fail
  - Upgrade node and rollup in github actions

### Added
  - Github action to check that all migrations run successfully
  - About page with organisation information

## [[0.9.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.7) - 2023-02-02
### Fixed
  - Correct wrong fields used in `0.9.6` migration

## [[0.9.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.6) - 2023-01-31
### Changed
  - Use inlined syntax in format strings to comply with [`rustc 1.67.0`](https://github.com/rust-lang/rust/releases/tag/1.67.0)
  - Upgrade rust to `1.67.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.4.0`, node `v18.13.0` and rollup `v3.12.0`) in production and development `Dockerfile`
  - [#457](https://github.com/thoth-pub/thoth/issues/457) - Upgrade `juniper` to v0.15.10
  - Upgrade `diesel` to v2.0.2
  - Upgrade `uuid` to v0.8.2
  - Upgrade `paperclip` to v0.8.0
  - Upgrade `graphql_client` to v0.12.0
  - Upgrade `chrono` to v0.4.23

### Fixed
  - [#469](https://github.com/thoth-pub/thoth/issues/469) - Expand DOI regex to include square brackets

## [[0.9.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.5) - 2023-01-17
### Changed
  - Upgrade rust to `1.66.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.2.0`, n `v9.0.1`, node `v18.12.1` and rollup `v3.7.4`) in production and development `Dockerfile`

### Fixed
  - [#463](https://github.com/thoth-pub/thoth/issues/463) - Update Thema codes to v1.5

## [[0.9.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.4) - 2022-12-05
### Added
  - [#414](https://github.com/thoth-pub/thoth/pull/414) - Synchronise chapters' `work_status` and `publication_date` with parent's upon parent's update

## [[0.9.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.3) - 2022-11-21
### Added
  - [#456](https://github.com/thoth-pub/thoth/pull/456) - Implement JSON output format

### Changed
  - [#455](https://github.com/thoth-pub/thoth/pull/455) - Extend CSV output format to include all available fields

## [[0.9.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.2) - 2022-11-01
### Changed
  - [#396](https://github.com/thoth-pub/thoth/pull/396) - Expand the list of contribution types with: SoftwareBy, ResearchBy, ContributionsBy, Indexer
  - [#451](https://github.com/thoth-pub/thoth/pull/451) - Output both short and long abstracts in Crossref DOI deposit

## [[0.9.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.1) - 2022-10-27
### Changed
  - [#449](https://github.com/thoth-pub/thoth/pull/449) - Update EBSCO Host ONIX price type code

## [[0.9.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.0) - 2022-10-24
### Added
  - [#333](https://github.com/thoth-pub/thoth/issues/333) - Add references to schema
  - Output references in Crossref DOI deposit
  - [#444](https://github.com/thoth-pub/thoth/issues/444) - Output abstracts in Crossref DOI deposit
  - [#443](https://github.com/thoth-pub/thoth/issues/443) - Output affiliations in Crossref DOI deposit
  - [#446](https://github.com/thoth-pub/thoth/issues/446) - Output fundings in Crossref DOI deposit

### Changed
  - Simplify syntax in CRUD methods

## [[0.8.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.11) - 2022-10-07
### Changed
  - [#298](https://github.com/thoth-pub/thoth/issues/298) - Make database constraint errors more user-friendly in API output and APP notifications
  - Replaced docker musl image (no longer maintained) with official images, installing requirements needed for static compilation

## [[0.8.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.10) - 2022-09-30
  - [#438](https://github.com/thoth-pub/thoth/issues/438) - Allow specifying query parameters based on the requested specification
  - Upgrade rust to `1.64.0` in development `Dockerfile`

## [[0.8.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.9) - 2022-09-21
### Added
  - [#426](https://github.com/thoth-pub/thoth/issues/426) - Add ProQuest Ebrary ONIX 2.1 specification
  - [#420](https://github.com/thoth-pub/thoth/issues/420) - Add RNIB Bookshare to the list of supported platforms for ONIX 2.1
  - [#423](https://github.com/thoth-pub/thoth/issues/423) - Add a link to the Thoth user manual under "Docs" tab of navbar
  - Development workflow in docker

### Changed
  - [#429](https://github.com/thoth-pub/thoth/issues/429) - Incomplete metadata record errors are now returned as a 404 instead of 500
  - Added derives for `Eq` alongside `PartialEq` to comply with [`rustc 1.63.0`](https://github.com/rust-lang/rust/releases/tag/1.63.0)
  - Upgrade rust to `1.63.0` in development `Dockerfile`
  - Order contributions and relations by ordinal, and subjects by type and ordinal

### Fixed
  - [#425](https://github.com/thoth-pub/thoth/issues/425) - Fix typo in contribution type illustrator
  - [#424](https://github.com/thoth-pub/thoth/issues/424) - Fix inactive tag on catalogue

## [[0.8.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.8) - 2022-08-02
### Added
  - [#389](https://github.com/thoth-pub/thoth/issues/389) - Streamline chapter (child work) creation process

### Changed
  - [#411](https://github.com/thoth-pub/thoth/issues/411) - Make `copyright_holder` optional
  - [#393](https://github.com/thoth-pub/thoth/issues/393) - Use en-dash in `page_interval` instead of hyphen
  - Ignore `extra_unused_lifetimes` warning until [clippy's fix](https://github.com/rust-lang/rust-clippy/issues/9014) for the false positive is live
  - Split build, test, and lint workflow job into separate jobs

## [[0.8.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.7) - 2022-07-22
### Fixed
  - [#379](https://github.com/thoth-pub/thoth/issues/379) - Limit to 6 the number of ISBNs offered in CrossRef metadata export
  - [#388](https://github.com/thoth-pub/thoth/issues/388) - Upgrade packages flagged in Dependabot alerts

### Changed
  - [#370](https://github.com/thoth-pub/thoth/issues/370) - Upgrade Yew to v0.19

## [[0.8.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.6) - 2022-07-01
### Added
  - [#390](https://github.com/thoth-pub/thoth/pull/390) - Implement OverDrive ONIX 3.0 specification

### Fixed
  - [#392](https://github.com/thoth-pub/thoth/issues/392) - Fix encoding of print ISBN in JSTOR ONIX output

## [[0.8.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.5) - 2022-05-30
### Added
  - [#287](https://github.com/thoth-pub/thoth/issues/287) - Allow editing contributions (and affiliations)

### Fixed
  - [#360](https://github.com/thoth-pub/thoth/issues/360) - Prevent adding 0 as the price of a publication
  - [#376](https://github.com/thoth-pub/thoth/issues/376) - Restrict Licence field entries to URL-formatted strings

## [[0.8.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.4) - 2022-05-11
### Added
  - [#29](https://github.com/thoth-pub/thoth/issues/29) - Implement CrossRef DOI Deposit specification
  - [#72](https://github.com/thoth-pub/thoth/issues/72) - Implement Google Books ONIX 3.0 specification

### Changed
  - [#356](https://github.com/thoth-pub/thoth/issues/356) - Upgrade actix to v4

## [[0.8.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.3) - 2022-04-18
### Added
  - [#359](https://github.com/thoth-pub/thoth/issues/359) - Allow editing publications

## [[0.8.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.2) - 2022-04-06
### Changed
  - Added CA certificates to docker image to allow https requests from containers

## [[0.8.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.1) - 2022-03-11
### Added
  - [#104](https://github.com/thoth-pub/thoth/issues/104) - Implement BibTeX specification

### Changed
  - Removed unnecessary title branching logic from KBART/ONIX output formats

## [[0.8.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.0) - 2022-03-01
### Added
  - [#341](https://github.com/thoth-pub/thoth/issues/341) - Add weight to publication

### Changed
  - Tidied verbose bools and single-character strings to comply with [`rustc 1.59.0`](https://github.com/rust-lang/rust/releases/tag/1.59.0)
  - [#300](https://github.com/thoth-pub/thoth/issues/300) - Moved width/height to Publication, added depth, improved metric/imperial display
  - Upgrade docker's base images to latest available releases

## [[0.7.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.2) - 2022-02-08
### Changed
  - [#339](https://github.com/thoth-pub/thoth/pull/339) - Update publication types to include AZW3, DOCX and FictionBook
  - [#331](https://github.com/thoth-pub/thoth/pull/331) - Update series model to include description and CFP URL
  - Allow triggering docker action manually

### Added
  - Add code of conduct and support document to repository

## [[0.7.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.1) - 2022-01-24
### Changed
  - Removed redundant `to_string` calls to comply with [`rustc 1.58.0`](https://github.com/rust-lang/rust/releases/tag/1.58.0)
  - [#329](https://github.com/thoth-pub/thoth/pull/329) - Update EBSCO Host ONIX pricing and contributor display logic
  - Allow building docker image manually in actions

## [[0.7.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.0) - 2022-01-11
### Added
  - [#28](https://github.com/thoth-pub/thoth/issues/28) - Implement chapter structure
  - GraphQL queries: support filtering on multiple enum variants (e.g. work types, language codes)
  - Dashboard: display Institution stats

### Fixed
  - Issues form: typing filter string in series search box has no effect on which series are displayed

## [[0.6.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.6.1) - 2021-12-13
### Changed
  - Removed redundant closures and `impl`s to comply with [`rustc 1.57.0`](https://github.com/rust-lang/rust/releases/tag/1.57.0)

### Fixed
  - [#309](https://github.com/thoth-pub/thoth/issues/309) - Update Thema codes to v1.4

## [[0.6.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.6.0) - 2021-11-29
### Added
  - [#92](https://github.com/thoth-pub/thoth/issues/92) - Implement institution table, replacing funder and standardising contributor affiliations

## [[0.5.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.5.0) - 2021-11-29
### Added
  - [#297](https://github.com/thoth-pub/thoth/issues/297) - Implement publication location

### Changed
  - Requirement to Number fields preventing user from entering numbers below 0 for Counts/below 1 for Editions and Ordinals, and sets Contribution Ordinal default to 1 instead of 0
  - [#299](https://github.com/thoth-pub/thoth/pull/299) - Update Project MUSE ONIX subject output logic
  - Updated if and else branches to comply with [`rustc 1.56.0`](https://github.com/rust-lang/rust/releases/tag/1.56.0)

### Fixed
  - [#292](https://github.com/thoth-pub/thoth/issues/292) - Cannot unset publication date: error when trying to clear a previously set publication date
  - [#295](https://github.com/thoth-pub/thoth/issues/295) - various subforms failing to trim strings before saving (including on mandatory fields which are checked for emptiness)
  - Factored out duplicated logic for handling optional field values, simplifying the code and reducing the likelihood of further bugs such as [#295](https://github.com/thoth-pub/thoth/issues/295) being introduced
  - Minor issue where some required fields were not marked as "required" (so empty values would be sent to the API and raise an error)
  - Issue with subforms where clicking save button bypassed field requirements (so instead of displaying a warning message such as "Please enter a number", invalid values would be sent to the API and raise an error)
  - [#310](https://github.com/thoth-pub/thoth/issues/310) - Add jstor specification to formats

## [[0.4.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.7) - 2021-10-04
### Added
  - [#43](https://github.com/thoth-pub/thoth/issues/43), [#49](https://github.com/thoth-pub/thoth/issues/49) - Implement EBSCO Host's ONIX 2.1 specification
  - [#44](https://github.com/thoth-pub/thoth/issues/44) - Implement JSTOR's ONIX 3.0 specification
  - [#253](https://github.com/thoth-pub/thoth/issues/253) - Implement Project MUSE ONIX specification tests

### Changed
  - [#242](https://github.com/thoth-pub/thoth/issues/242) - Move API models to object-specific subdirectories
  - [#274](https://github.com/thoth-pub/thoth/issues/274) - Add width/height units to CSV specification
  - [#263](https://github.com/thoth-pub/thoth/issues/263) - Add `Doi`, `Isbn` and `Orcid` types to client schema

## [[0.4.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.6) - 2021-09-02
### Added
  - [#88](https://github.com/thoth-pub/thoth/issues/88) - Implement KBART specification
  - [#266](https://github.com/thoth-pub/thoth/issues/266) - Delete confirmation to publications

### Changed
  - [#272](https://github.com/thoth-pub/thoth/issues/272) - Use more fields in `contributors` filtering

### Fixed
  - [#271](https://github.com/thoth-pub/thoth/issues/271) - Make filter parameter optional in `subjectCount`

## [[0.4.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.5) - 2021-08-12
### Added
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Units selection dropdown to Work and NewWork pages, which updates the Width/Height display on change
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Local storage key to retain user's choice of units across all Work/NewWork pages
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Backend function to convert to/from database units (mm): uses 1inch = 25.4mm as conversion factor, rounds mm values to nearest mm, rounds cm values to 1 decimal place, rounds inch values to 2 decimal places
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Constraints on Width/Height fields depending on unit selection: user may only enter whole numbers when in mm, numbers with up to 1 decimal place when in cm, numbers with up to 2 decimal places when in inches

### Changed
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - GraphQL and APP queries to specify units when submitting new Width/Height values, and handle conversion if required

## [[0.4.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.4) - 2021-08-02
### Fixed
  - Read button in catalogue now uses the landing page URL instead of the DOI

### Changed
  - Removed needless borrow to comply with `clippy` under [`rustc 1.54.0`](https://github.com/rust-lang/rust/releases/tag/1.54.0)

## [[0.4.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.3) - 2021-07-28
### Added
  - [#48](https://github.com/thoth-pub/thoth/issues/48) - Implement OAPEN ONIX 3.0 specification

### Fixed
  - [#254](https://github.com/thoth-pub/thoth/issues/254) - Ensure order of fields in create work match those in edit work

## [[0.4.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.2) - 2021-07-05
### Added
  - [#125](https://github.com/thoth-pub/thoth/issues/125) - Implement `ISBN` type to standardise parsing
  - [#217](https://github.com/thoth-pub/thoth/issues/217) - Add "Contribution Ordinal" field to indicate order of contributions within a work

## [[0.4.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.1) - 2021-06-22
### Changed
  - [#234](https://github.com/thoth-pub/thoth/issues/234) - Move database calls out of GraphQL model

### Added
  - [#136](https://github.com/thoth-pub/thoth/issues/135), [#233](https://github.com/thoth-pub/thoth/issues/233) - Implement `Doi` and `Orcid` types to standardise parsing
  - `thoth-errors` crate to share `ThothError` and `ThothResult`

## [[0.4.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.0) - 2021-06-15
### Changed
  - Updated `yew` to [`v0.18.0`](https://github.com/yewstack/yew/releases/tag/0.18.0)
  - Updated `actix-web` to [`3.3.2`](https://github.com/actix/actix-web/releases/tag/web-v3.3.2)
  - Catch client errors with `ThothError::EntityNotFound`
  - Use a custom instance of GaphiQL
  - Unify `Work` output structure in client using GraphQL fragments

### Added
  - [#235](https://github.com/thoth-pub/thoth/issues/235) - Export API with openapi schema
  - [#110](https://github.com/thoth-pub/thoth/issues/110) - Output to CSV
  - Rapidoc schema explorer interface

### Removed
  - `actix_rt`

## [[0.3.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.6) - 2021-05-11
### Fixed
  - Problem building docker image

## [[0.3.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.5) - 2021-05-11
### Added
  - [#213](https://github.com/thoth-pub/thoth/issues/213) - Link to documentation in readme
  - [#206](https://github.com/thoth-pub/thoth/issues/206) - Notify user when a new version of the APP is available
  - [#231](https://github.com/thoth-pub/thoth/issues/231) - Link to publication page in work page
  - [#224](https://github.com/thoth-pub/thoth/issues/224) - Implement limit and offset in linked queries
  - Implement Crud trait with database calls per object

### Changed
  - [#236](https://github.com/thoth-pub/thoth/issues/236) - Split server logic into individual crates
  - Update rustc to 1.51.0 in docker image
  - Replace composite keys in `contribution` and `issue` with standard UUIDs
  - Server configuration parsed from binary

### Fixed
  - [#216](https://github.com/thoth-pub/thoth/issues/216), [#228](https://github.com/thoth-pub/thoth/issues/228) - Error adding multiple subjects


## [[0.3.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.4) - 2021-03-29
### Fixed
  - Upgraded rusct in docker image. Moved `wasm-pack` to a less fragile build stage using official image, keeping main build statically compiled

## [[0.3.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.3) - 2021-03-26
### Added
  - [#120](https://github.com/thoth-pub/thoth/issues/120) - Implement table sorting by columns in APP
  - [#203](https://github.com/thoth-pub/thoth/issues/203) - Cascade filtering options to relation queries in API

### Changed
  - [#210](https://github.com/thoth-pub/thoth/issues/210) - Specify .xml extension when outputting ONIX files

### Fixed
  - [#182](https://github.com/thoth-pub/thoth/issues/182) - Ensure issue's series and work have the same imprint


## [[0.3.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.2) - 2021-03-09
### Added
  - [#202](https://github.com/thoth-pub/thoth/issues/202) - Enum type filtering in GraphQL queries
  - [#202](https://github.com/thoth-pub/thoth/issues/202) - Query works by DOI
  - [#195](https://github.com/thoth-pub/thoth/issues/195) - Prompt confirmation upon delete

### Fixed
  - [#199](https://github.com/thoth-pub/thoth/issues/199), [#201](https://github.com/thoth-pub/thoth/issues/201) - Error displaying publications if filtering on empty ISBN or URL
  - Trigger a warning when the current user does not have any editting permissions

## [[0.3.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.1) - 2021-03-04
### Fixed
  - [#197](https://github.com/thoth-pub/thoth/issues/197) - Error deserialising publications in APP

## [[0.3.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.0) - 2021-03-03
### Changed
  - [#162](https://github.com/thoth-pub/thoth/issues/162) - Only records linked to publishers user has access to are listed in APP
  - [#167](https://github.com/thoth-pub/thoth/issues/167) - Make work contribution the canonical source of contributor names in ONIX output

### Added
  - [#177](https://github.com/thoth-pub/thoth/issues/177) - Allow querying objects by linked publisher(s)
  - [#159](https://github.com/thoth-pub/thoth/issues/159), [#160](https://github.com/thoth-pub/thoth/issues/160), [#161](https://github.com/thoth-pub/thoth/issues/161) - Add publisher accounts
  - [#163](https://github.com/thoth-pub/thoth/issues/163) - Save a snapshot of each object upon update
  - [#164](https://github.com/thoth-pub/thoth/issues/164), [#165](https://github.com/thoth-pub/thoth/issues/165) - Add contributor names to contribution
  - [#168](https://github.com/thoth-pub/thoth/issues/168) - Warn users when editing a contributor or a funder that is linked to a work
  - [#185](https://github.com/thoth-pub/thoth/issues/185) - Allow resetting user passwords through CLI
  - Allow creating publisher accounts through CLI

### Fixed
  - [#181](https://github.com/thoth-pub/thoth/issues/181) - Enforce numeric values for issue ordinal

## [[0.2.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.13) - 2021-01-14
### Changed
  - Update API URL in docker github action
  - Remove staging tag in docker github action

## [[0.2.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.12) - 2021-01-12
### Changed
  - [#153](https://github.com/thoth-pub/thoth/issues/153) - Implement created and updated dates to each structure

## [[0.2.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.11) - 2021-01-06
### Changed
  - [#151](https://github.com/thoth-pub/thoth/issues/151) - Make browser prompt user to save Onix XML to file
  - [#143](https://github.com/thoth-pub/thoth/issues/143) - Start using Github Actions instead of Travis

### Added
  - [#121](https://github.com/thoth-pub/thoth/issues/121) - Add created and updated dates to each table

## [[0.2.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.10) - 2021-01-04
### Changed
  - [#127](https://github.com/thoth-pub/thoth/issues/127) - Do not exit main entity edit pages upon saving
  - [#147](https://github.com/thoth-pub/thoth/issues/147) - Remove subject code validation for non open subject headings

## [[0.2.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.9) - 2020-11-24
### Changed
  - Hide creative commons icon when license is unset in APP catalogue

### Added
  - Display book cover placeholder when cover URL is unset
  - Status tags to APP catalogue

## [[0.2.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.8) - 2020-11-23
### Changed
  - Upgrade fontawesome to v5.4.0

### Added
  - Information banner to APP homepage
  - New BISAC codes

## [[0.2.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.7) - 2020-11-19
### Changed
  - [#118](https://github.com/thoth-pub/thoth/issues/118) - Ensure empty data is sent as null not as empty strings
  - [#131](https://github.com/thoth-pub/thoth/issues/131) - Moved forms with relationships outside main object form

## [[0.2.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.6) - 2020-11-13
### Changed
  - Fix pricing functionality ommitted in previous release

## [[0.2.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.5) - 2020-11-13
### Added
  - New BISAC codes

## [[0.2.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.4) - 2020-11-10
### Added
  - Implemented pricing CRUD in APP

## [[0.2.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.3) - 2020-11-06
### Added
  - Implemented pagination in all admin components
  - Implemented pagination in catalogue

## [[0.2.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.2) - 2020-11-03
### Changed
  - Set `THOTH_API` on build via docker

## [[0.2.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.1) - 2020-11-02
### Changed
  - Redirect to relevant routes upon save and create actions in APP

### Added
  - Delete functionality in all APP objects

## [[0.2.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.0) - 2020-10-23
### Changed
  - [#38](https://github.com/thoth-pub/thoth/issues/38) - Split client and server
  - [#98](https://github.com/thoth-pub/thoth/issues/98) - Streamline Thoth logo

### Added
  - [#97](https://github.com/thoth-pub/thoth/issues/97), [#39](https://github.com/thoth-pub/thoth/issues/39), [#41](https://github.com/thoth-pub/thoth/issues/41) - Implement WASM frontend with Yew
  - [#40](https://github.com/thoth-pub/thoth/issues/40) - Implement API authentication

## [[0.1.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.10) - 2020-06-03
### Changed
  - Roadmap button in index catalogue

## [[0.1.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.9) - 2020-06-03
### Added
  - Roadmap document

## [[0.1.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.8) - 2020-06-02
### Changed
  - New design for the index catalogue

## [[0.1.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.7) - 2020-03-27
### Changed
  - [#35](https://github.com/thoth-pub/thoth/issues/35) - Fix date format and lack in ONIX sender header
  - Add place of publication to ONIX file
  - Use code 03 (description) instead of 30 (abstract) in OAPEN ONIX

## [[0.1.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.6) - 2020-03-26
### Changed
  - Fix incompatibilities with OAPEN ONIX parser
  - Map ONIX parameter to UUID directly, instead of converting afterwards
  - Normalise server route definitions

## [[0.1.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.5) - 2020-03-25
### Changed
  - Load assets statically

## [[0.1.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.4) - 2020-03-24
### Changed
  - "/" now renders its own page, instead of redirecting to "/graphiql"
  - [#27](https://github.com/thoth-pub/thoth/issues/27) - Produce an OAPEN compatible ONIX file

### Added
  - [#26](https://github.com/thoth-pub/thoth/issues/26) - Create an endpoint to allow generating ONIX streams from "/onix/{workId}"

### Removed
  - Dropped support for creating ONIX from binary

## [[0.1.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.3) - 2020-03-16
### Changed
  - Pin compiler's docker image to a specific version (best practice)
  - Use COPY instead of ADD for directories in Dockerfile (best practice)
  - [#24](https://github.com/thoth-pub/thoth/issues/24) - Implemented rust style guidelines

### Added
  - [#23](https://github.com/thoth-pub/thoth/issues/23) - Redirect "/" to "/graphiql"
  - [#18](https://github.com/thoth-pub/thoth/issues/18) - Create ThothError structure to start catching all other types of errors
  - [#24](https://github.com/thoth-pub/thoth/issues/24) - Enforce rust style guidelines using husky (pre-push hook) and travis
  - [#17](https://github.com/thoth-pub/thoth/issues/17) - Allow producing a proto ONIX file from the binary

## [[0.1.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.2) - 2020-03-03
### Changed
  - [#10](https://github.com/thoth-pub/thoth/issues/10) - Port exposing is handled in Dockerfile instead of docker-compose
  - [#16](https://github.com/thoth-pub/thoth/issues/16) - Moved server start function from binary to library
  - [#9](https://github.com/thoth-pub/thoth/issues/9) - Docker image is now compiled statically

### Added
  - [#13](https://github.com/thoth-pub/thoth/issues/13) - Added limit and offset arguments to all queries
  - [#13](https://github.com/thoth-pub/thoth/issues/13) - Added default order by clauses to all queries
  - [#15](https://github.com/thoth-pub/thoth/issues/15) - Implemented GraphQL errors for diesel errors
  - [#13](https://github.com/thoth-pub/thoth/issues/13) - Added filter arguments for publishers and works queries

## [[0.1.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.1) - 2020-02-27
### Changed
  - Improved Dockerfile to allow running database migrations at run time

### Added
  - Implemented imprints for publisher graphql object
  - [#6](https://github.com/thoth-pub/thoth/issues/6) - Added subcommands to main binary to allow running embedded migrations without having to install diesel\_cli
  - Automatic publication to crates.io

## [[0.1.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.0) - 2020-02-21
### Added
  - Database migrations
  - GraphQL handlers implementing the thoth schema
