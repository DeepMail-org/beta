# `frontend-beta/src/lib` Folder Guide

This folder contains API helpers, runtime contracts, formatting helpers, and shared utility code.

## Contents

| Path | Purpose |
| --- | --- |
| `api.ts` | Backend client helpers and token storage helpers. |
| `types.ts` | Shared frontend TypeScript types. |
| `format.ts` | Shared display-formatting helpers. |
| `format.test.ts` | Unit tests for formatting helpers. |
| `utils.ts` | Miscellaneous shared frontend utilities. |
| `contracts/` | Runtime validation schemas for backend payloads. |

## Key Files And Functions

- `api.ts`
  - `ApiError`: standard error wrapper for API failures.
  - `getToken()`: reads the saved client token.
  - `setToken()`: writes the saved client token.
  - `clearToken()`: removes the saved client token.
- `format.ts`: presentation helpers for user-facing values.
- `contracts/results.ts`: runtime contract for analysis result payloads.

## Read Next

- `contracts/README.md`
