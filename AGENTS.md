# Repository Instructions

- Keep `README.md` aligned with the current runnable app surface. When controls, pages, or major workflow behavior change, update the README in the same change.
- Keep the tracked renderer-level screenshots current when the main screens change. The tracked screenshots are:
  - `artifacts/screenshots/timeline.png`
  - `artifacts/screenshots/mappings.png`
  - `artifacts/screenshots/mappings-overlay.png`
  - `artifacts/screenshots/midi-io.png`
  - `artifacts/screenshots/routing.png`
- Regenerate screenshots with:
  - `powershell -ExecutionPolicy Bypass -File .\scripts\capture-ui-screens.ps1 -StateMode demo`
- Refresh the latest visual review when UI layout changes materially:
  - `powershell -ExecutionPolicy Bypass -File .\scripts\review-ui-screens.ps1 -StateMode demo`
- Commit the latest renderer-owned screenshots and `artifacts/reviews/ui-findings.md` when they are intentionally refreshed.
- Do not commit ephemeral state or archives:
  - `artifacts/archive/`
  - `artifacts/state/`
  - `docs/artifacts/`
  - `scripts/artifacts/`
