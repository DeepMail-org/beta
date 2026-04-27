# `frontend-beta/src/app` Folder Guide

This folder owns the Next.js route tree.

## Contents

| Path | Purpose |
| --- | --- |
| `layout.tsx` | Root layout component for all routes. |
| `page.tsx` | Landing/dashboard entry page. |
| `analysis/` | Analysis list and detail routes. |
| `graph/` | Top-level graph visualization route. |
| `reports/` | Report listing route. |
| `sandbox/` | Sandbox results route. |
| `settings/` | Token/settings route. |
| `upload/` | Upload route. |
| `favicon.ico` | Browser favicon asset bundled through the app tree. |

## Key Files And Functions

- `layout.tsx`
  - `RootLayout`: wraps all pages with shared document structure.
- `page.tsx`
  - route component for the primary entry screen.

## Read Next

- `analysis/README.md`
- route-folder READMEs for the view you are changing
