# `frontend-beta/tests/contract` Folder Guide

This folder contains contract tests that validate frontend assumptions about backend responses.

## Files

| File | Purpose |
| --- | --- |
| `results.contract.test.ts` | Verifies that the frontend analysis-result contract matches the payload shape it expects to render. |

## Editing Guidance

- Update these tests whenever `src/lib/contracts/results.ts` changes.
