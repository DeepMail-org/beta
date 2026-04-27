# `frontend-beta/src` Folder Guide

This folder contains all application source code for the frontend prototype.

## Contents

| Path | Purpose |
| --- | --- |
| `app/` | Next.js App Router entrypoints and route trees. |
| `components/` | Shared UI and feature components. |
| `hooks/` | Reusable React hooks. |
| `lib/` | API helpers, contracts, formatting, and utility functions. |
| `globals.css` | Global styles and shared theme rules. |

## Key Files And Functions

- `globals.css`: application-wide visual theme and shared CSS rules.
- `app/layout.tsx`: root application shell.
- `app/page.tsx`: top-level landing/dashboard route.

## How To Read This Folder

1. Start with `app/README.md` for route ownership.
2. Read `components/README.md` for UI composition.
3. Read `lib/README.md` for API and data-contract assumptions.
