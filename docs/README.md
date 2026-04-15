# Documentation Map

This folder is organized by audience and intent:

- `docs/specs/` — product and feature specifications. Use these when defining behavior, UX, and acceptance criteria.
- `docs/dev/` — implementation-facing technical references (architecture notes, mapping references, etc.).
- `docs/planning/` — plans, handoffs, and roadmap/work breakdown documents.
- `docs/user/` — end-user guides, tutorials, and operator-facing docs.

Recent/additional specs:

- `docs/specs/ui-scaling-spec.md` — current implemented UI scaling behavior and constraints.
- `docs/specs/feature-spec-quick-mapping-lookup.md` — target-field fuzzy lookup/edit flow for the mappings page.

## Contribution rules

When adding or updating docs:

1. Put the doc in the folder that matches its primary audience.
2. Keep filenames descriptive and stable; prefer updating existing docs over creating near-duplicates.
3. Cross-link related docs using repo-root-relative paths (for example, `docs/specs/product-spec.md`).
4. For behavior changes, update specs (`docs/specs/`) and implementation/planning docs (`docs/dev/`, `docs/planning/`) together as needed.
5. Keep root-level `README.md` and tracked UI artifacts aligned with the current app surface per `AGENTS.md`.

If a document no longer fits its folder, move it with `git mv` to preserve history where feasible.
