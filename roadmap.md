# Thoth roadmap

The purpose of this document is to reflect the present state of thoth to serve as the basis for a discussion about the requirements needed and the steps we need to take to get there.

## Current functionality

Thoth currently consists of the following:

  - Metadata schema
  - GraphQL API that implements the schema
  - OAPEN ONIX output
  - HTML Catalogue

All the above is run using a single binary file that starts the GraphQL backend and can deliver the HTML catalogue (which consumes the API). Only two containers are needed for deployment: the thoth backend and a PostgreSQL database.

At present the whole system lacks any authentication mechanism, which is not needed for testing purposes, but will definitely be needed when used in production.

The main flaw right now has to do with metadata quality and its ingestion process. Ingestion is done using [thoth-loader](https://github.com/openbookpublishers/thoth-loader/), a script that reads from bespoke CSV files and converts the data to GraphQL mutations that are triggered on thoth to create new records. This process can be fiddly in production as updates are not currently handled, i.e. we can only create new records, and not update new ones. The CSV files used are also outdated and do not contain good quality data.

## Short to medium term roadmap

Overall we have two paths to follow: either we focus on developing new features, new metadata outputs that we have already identified as necessary; or we try to consolidate what we already have to then expand as necessary.

We should probably opt for the latter, a more agile approach in which we get thoth to a stable version with minimal functionality that could be used in production, and then we make further revisions adding functionality.

In order to get to that working prototype we need:
  1. to separate the server functionality (API) from the client (exports: catalogue, ONIX, etc.);
  2. create an admin interface, separate from the public catalogue;
  3. add bulk metadata ingest functionality to thoth;
  4. add authentication to both the API and the GUI.

Following the path of developing new outputs is also a valid option, which would allows us to keep adding strength to our proof of concept.

