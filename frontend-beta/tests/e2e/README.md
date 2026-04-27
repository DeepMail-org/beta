# `frontend-beta/tests/e2e` Folder Guide

This folder contains Playwright end-to-end tests for user-critical frontend flows.

## Files

| File | Purpose |
| --- | --- |
| `map.spec.ts` | Verifies map interaction, large-dataset rendering, and retry handling. |
| `README.md` | Documents the end-to-end test folder and how it is used. |

## Current Coverage

- marker click opens the enriched sidebar
- clustered-map datasets render safely for larger result sets
- retry flow recovers from API error state

## Run

```bash
bun run test:e2e
```

## Notes

- Tests stub `/api/v1/results/:email_id` using Playwright route handlers.
- The tests validate the current REST-style prototype contract, not the final planned GraphQL gateway contract.
