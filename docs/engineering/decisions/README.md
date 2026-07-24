# Engineering Decisions

Status: ACTIVE decision process
Owner: CTO

This directory contains cross-programme Architecture Decision Records and their normative appendices.

## Authority

An ADR is authoritative only when its status is `APPROVED`.

Statuses:

- `PROPOSED` - complete recommendation awaiting the named decision owner;
- `APPROVED` - implementation may rely on the decision;
- `SUPERSEDED` - replaced by another approved ADR;
- `REJECTED` - considered and not adopted.

Committing a proposed ADR does not approve it.

## Current decisions

See `decision-register.md`.

## Numbering

Use:

```text
ADR-NNNN-short-title.md
```

Numbers are repository-wide and never reused.

## Required content

Each ADR must identify:

- decision owner;
- affected programmes and repositories;
- context and decision drivers;
- options considered;
- exact decision;
- invariants;
- implementation impact;
- migration, rollout and rollback effects;
- validation evidence;
- approval state.

## Cross-programme rule

A programme conversation may propose a cross-programme decision, but only the CTO control process may approve it.

Implementation must stop when it depends on a proposed decision that has not been approved.

## Amendments

Before implementation starts, a proposed ADR may be edited in place.

After approval:

- material architectural changes require a new ADR that supersedes the old one;
- factual clarifications may update the ADR only when they do not alter the decision;
- every update follows normal review and changelog requirements.
