# `.claude` Folder Guide

This folder stores generated planning artifacts and documentation snapshots used to reason about DeepMail's target architecture before the full service implementation exists.

## Contents

| File | Purpose |
| --- | --- |
| `deepmail_architecture.html` | Visual architecture reference describing layers, services, pipeline flow, storage, security, and infrastructure decisions. |
| `deepmail_schema_complete.html` | Visual schema reference describing the proposed database layout for backend services. |

## How This Folder Fits Into DeepMail

- It is a planning and review area, not a runtime dependency.
- It helps humans and agents compare intended architecture against actual code in `crates/` and `frontend-beta/`.
- When docs and code disagree, treat this folder as a design artifact that may need reconciliation rather than as unquestionable truth.

## Editing Guidance

- Keep documents consistent with the repository state and the canonical DeepMail platform specification.
- Do not store secrets, credentials, or local environment dumps here.
- If a new planning document is added, update this README so future readers know why it exists.
