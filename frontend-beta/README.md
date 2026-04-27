# `frontend-beta` Folder Guide

`frontend-beta` is the current Next.js 16 prototype frontend for DeepMail.

## Purpose

This folder contains the most feature-rich user-facing surface in the repository today. It covers landing pages, upload and analysis routes, graph and map visualizations, API helpers, and frontend contract tests.

## Important Architectural Note

The prototype currently talks to backend endpoints as a REST-style client and stores a token in browser storage. That does not yet match the long-term DeepMail gateway plan, which is intended to be GraphQL-first with stricter auth handling. Treat the frontend as a working prototype with known architectural drift.

## Contents

| Path | Purpose |
| --- | --- |
| `src/` | Application source code for routes, components, hooks, and utilities. |
| `public/` | Static assets bundled by Next.js. |
| `tests/` | Contract and end-to-end frontend tests. |
| `package.json` | Frontend package manifest and scripts. |
| `bun.lock` | Bun lockfile for deterministic installs. |
| `next.config.ts` | Next.js runtime configuration. |
| `tsconfig.json` | TypeScript compiler settings. |
| `eslint.config.mjs` | ESLint configuration. |
| `postcss.config.mjs` | PostCSS configuration for styling. |
| `components.json` | UI component metadata/config used by the prototype toolchain. |

## Key Files And Functions

- `src/app/layout.tsx`
  - `RootLayout`: global HTML shell for the app.
- `src/app/page.tsx`
  - page component for the landing/dashboard entry surface.
- `src/lib/api.ts`
  - `ApiError`: API-specific error wrapper.
  - `getToken()`, `setToken()`, `clearToken()`: client-side token helpers.
- `src/lib/format.ts`
  - formatting helpers used across UI rendering.
- `src/lib/contracts/results.ts`
  - runtime validation for analysis result payloads.
- `src/hooks/useThreatGraph.ts`
  - shared graph-view data orchestration hook.

## Development Commands

```bash
bun install
bun run dev
bun run lint
bun run build
bun run test:e2e
bun run test:contract
```

## Read Next

- `src/README.md`
- `tests/README.md`
