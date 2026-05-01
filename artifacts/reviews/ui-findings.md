Findings

1. severity: medium; screenshot/page name: `timeline.png`; issue: main timeline is visually dense with many similarly weighted panels (track headers, song lanes, loop lanes, FX rows), so scan order and “what to act on first” is unclear; brief suggested fix: increase hierarchy by making primary editable region brighter/thicker and de-emphasizing secondary metadata rows (lower contrast, smaller labels, or more spacing between sections).

2. severity: medium; screenshot/page name: `timeline-focused.png`; issue: focused mode still shows many compact top controls with equal emphasis, so “focused track” state is not immediately obvious beyond the small `TRACK T1` chip; brief suggested fix: add a stronger focused-state treatment (clear page subtitle, highlighted track header band, or dim non-focused controls).

3. severity: low; screenshot/page name: `mappings.png`; issue: footer hints (`SHIFT+LEFT/RIGHT FIELD`, `Q/E ADJUST`, `ENTER LEARN/TOGGLE`) are cramped and read like one continuous string, reducing quick comprehension; brief suggested fix: group hints with separators/padding and align by action category (navigation, edit, learn).

4. severity: low; screenshot/page name: `mappings-overlay.png`; issue: top-right status block (`ROWS 1-19 / 30`, `SCOPE`) is detached from table header and easy to miss; brief suggested fix: anchor row/scope status directly in the table header row or give it a boxed header treatment tied to the grid.

5. severity: medium; screenshot/page name: `midi-io.png`; issue: cards look selectable but there is no obvious active/focus indicator beyond subtle border color, and `DEF/SEL` tags are tiny, making state communication weak; brief suggested fix: use clearer selected/default badges (larger, higher-contrast) and add explicit active card highlight/fill treatment.

6. severity: medium; screenshot/page name: `routing.png`; issue: terminology and toggle semantics are ambiguous (`REC FX DRY`, `MON FX ON`, repeated `TGL` buttons), so it is hard to predict outcome quickly; brief suggested fix: replace shorthand with clearer labels (`Record FX Mode`, `Monitor FX`) and use explicit toggle controls with current-state text inside the control.

7. severity: low; screenshot/page name: `routing.png`; issue: spacing between left `SIGNAL` column and right `REC/MON` + FX blocks is tight relative to internal panel complexity, causing visual crowding; brief suggested fix: add more gutter space between major columns and slightly increase vertical rhythm within right-side control rows.

8. severity: low; screenshot/page name: `mappings.png` and `routing.png`; issue: some small labels/buttons near right edges (`ON`, `SET`, `TGL`) feel close to borders, which reads as near-clipping at a glance; brief suggested fix: add a few pixels of inner right padding and minimum button width for short labels.