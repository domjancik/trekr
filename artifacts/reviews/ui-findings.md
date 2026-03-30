Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** The dense multi-track view makes key labels (track names, lane numbers, loop metadata) hard to parse at a glance; header text and per-track controls are visually cramped. | **brief suggested fix:** Increase minimum column width or reduce simultaneous columns; prioritize larger type/spacing for track header and active-state rows.

2. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** Left and right panes use similar visual weight, so it is not immediately clear which pane is primary for editing vs reference. | **brief suggested fix:** Strengthen hierarchy with clearer pane titles and stronger active-pane treatment (contrast, border weight, or header emphasis).

3. **severity: low** | **screenshot/page:** `timeline.png` and `timeline-focused.png` | **issue:** Top-right status chips (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`, etc.) are tightly packed and read like one continuous string. | **brief suggested fix:** Add more horizontal spacing/grouping and use separators or boxed clusters by function.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Table density is very high; `TYPE / DEVICE / SOURCE / TARGET` rows are difficult to scan quickly, and right-side `SCOPE`/`ON` columns look detached from row meaning. | **brief suggested fix:** Increase row height/padding slightly and visually group “scope + enabled” with the main action cell (or add clearer column dividers).

5. **severity: low** | **screenshot/page:** `mappings.png` and `mappings-overlay.png` | **issue:** Row count/status text (`ROWS 1 / 30`, `ROWS 1-19 / 30`, `SCOPE`) is small and weak in hierarchy versus other chrome. | **brief suggested fix:** Promote these to a clearer status bar style with stronger contrast and consistent placement.

6. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are understated and easy to miss, which weakens state communication for “overlay mode.” | **brief suggested fix:** Add a more explicit overlay header band with stronger mode label and primary actions.

7. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards contain large unlabeled gray blocks that look like missing content/placeholders; it is unclear what each block represents. | **brief suggested fix:** Add explicit sublabels (e.g., channels/activity/notes) or empty-state labels; reduce blank area if no secondary data is shown.

8. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Repeated small `SET`/`TGL` controls and +/- steppers create ambiguous action semantics (edit value vs apply vs toggle). | **brief suggested fix:** Standardize control patterns per row (e.g., value editor + single apply action) and add concise inline labels/tooltips for control intent.