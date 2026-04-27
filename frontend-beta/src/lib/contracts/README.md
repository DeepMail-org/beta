# `frontend-beta/src/lib/contracts` Folder Guide

This folder contains runtime validation rules for backend payloads consumed by the frontend.

## Files

| File | Purpose |
| --- | --- |
| `results.ts` | Runtime validation schema for analysis result payloads. |

## Key Files And Functions

- `results.ts`: defines the contract the frontend expects from backend result endpoints before rendering maps, graphs, and analysis views.

## Editing Guidance

- Update this folder when backend response shapes change.
- Keep tests in sync under `tests/contract/`.
