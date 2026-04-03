Findings

1. **severity: medium** — **screenshot/page:** `timeline.png` — **issue:** Information density is very high in vertical mode; six narrow track columns compress labels and make row/loop context hard to scan quickly. **brief suggested fix:** Reduce simultaneous columns (or add stronger per-track grouping), and increase minimum text/cell width for headers and key states.

2. **severity: medium** — **screenshot/page:** `timeline.png` — **issue:** Top-right status cluster (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`, `LAUNCHQ OFF`, `LAUNCH BAR`, `QUANT 1/16`, `PEERS`) has weak hierarchy and reads like one long strip of similar-weight tokens. **brief suggested fix:** Split into labeled groups (transport, launch, quantize, peers) with clearer spacing and stronger active/inactive contrast.

3. **severity: low** — **screenshot/page:** `timeline-focused.png` — **issue:** The focused view still contains many small controls at the top, so “focused track” state is not strongly communicated despite the mode change. **brief suggested fix:** Emphasize focused mode with a larger mode banner and de-emphasize non-focused global controls.

4. **severity: medium** — **screenshot/page:** `mappings.png` — **issue:** Table columns are crowded; trigger strings and scope values (for example compact scope labels) are hard to parse at speed, and row semantics rely on very subtle separators. **brief suggested fix:** Increase column padding/contrast, widen trigger/scope columns, and add stronger zebra striping or row grouping.

5. **severity: low** — **screenshot/page:** `mappings-overlay.png` — **issue:** Overlay shortcut hints (`F5 CLOSE`, `W WRITE`) are visually similar to table content, so primary action vs data is ambiguous. **brief suggested fix:** Move overlay actions into a distinct header bar style (badge/button treatment) separated from table headings.

6. **severity: medium** — **screenshot/page:** `midi-io.png` — **issue:** Large empty device panes dominate the screen while actual actionable controls/states (`DEF`, `SEL`) are tiny and easy to miss. **brief suggested fix:** Promote selection/default states with larger labels and reduce empty panel weight (or add explicit “no details” placeholders).

7. **severity: medium** — **screenshot/page:** `routing.png` — **issue:** Many `SET`/`TGL` controls look nearly identical across different contexts (input, rec/mon, output), which makes action intent unclear. **brief suggested fix:** Add context-specific button labels/icons (e.g., `SET INPUT CH`, `TOGGLE MON`) and stronger section-level visual differentiation.

8. **severity: low** — **screenshot/page:** `routing.png` — **issue:** Mixed spacing and alignment between stacked cards (left signal chain vs right fx cards) creates slight visual imbalance and slows scanning. **brief suggested fix:** Normalize vertical rhythm and align card baselines/padding across columns.