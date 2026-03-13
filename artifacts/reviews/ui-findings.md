Findings

1. severity: medium | screenshot/page: `mappings.png` | issue: Table density is very high (tight row height + minimal column padding), so rows blend together and scanning quickly is hard. | brief suggested fix: Increase row height/padding slightly and add stronger zebra/hover contrast so each mapping row is easier to parse.

2. severity: medium | screenshot/page: `mappings.png` | issue: Bottom shortcut legend is cramped and uses terse labels (`TAP AGAIN ACT`, `0/E ADJUST`) that are hard to decode at a glance. | brief suggested fix: Group shortcuts by function and use clearer labels (or short tooltips/help text) with more spacing between groups.

3. severity: low | screenshot/page: `mappings-overlay.png` | issue: Header metadata on the right (`ROWS 1-19 / 30`, `SCOPE`) feels mis-grouped and visually detached from the table columns. | brief suggested fix: Align these with the column header row or place them in a compact single status line above the table.

4. severity: medium | screenshot/page: `mappings-overlay.png` | issue: The overlay leaves a large unused empty area below the visible rows, which weakens information hierarchy and makes content feel unfinished. | brief suggested fix: Either show more rows in the viewport or reduce panel height to fit current content density.

5. severity: medium | screenshot/page: `midi-io.png` | issue: Device cards contain large unlabeled gray blocks that look like missing content/placeholders rather than meaningful state. | brief suggested fix: Add explicit labels/state text inside those regions (e.g., channels, activity, or “no signal”) or reduce their visual weight.

6. severity: low | screenshot/page: `midi-io.png` | issue: State chips (`DEF`, `SEL`) are very small and tucked into corners, so selection/default status is easy to miss. | brief suggested fix: Increase chip size/contrast and place state near the device title with consistent spacing.

7. severity: medium | screenshot/page: `routing.png` | issue: Right-side per-row actions (`+`, `SELECT`, `TAP +/-`) are ambiguous and visually similar, making control intent unclear. | brief suggested fix: Replace symbol-only actions with explicit labels/icons plus short hints (e.g., `Next`, `Pick`, `Tap Learn`).

8. severity: medium | screenshot/page: `timeline.png` | issue: Top control strips are overcrowded (many compact toggles and status tokens in one line), which weakens hierarchy and makes mode/state hard to read quickly. | brief suggested fix: Split controls into grouped sections (transport, loop, quantize/link) with clearer spacing and stronger active-state emphasis.

9. severity: medium | screenshot/page: `timeline.png` and `timeline-focused.png` | issue: Track headers and loop detail labels are very small relative to grid area, so track identity and state changes are not immediately legible. | brief suggested fix: Increase header typography/contrast and give active track/state badges more visual priority than grid chrome.