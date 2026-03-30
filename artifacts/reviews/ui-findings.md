Findings

1. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Multiple labels are visibly truncated (`TRA...`, `VEL...`) in the Input/Output FX cards, which hides meaning and forces guessing. | **brief suggested fix:** Widen those fields or use compact but explicit abbreviations with tooltips/full text on focus.

2. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** The top control rows are very dense (`WRAP EXTEND`, `SONG LOOP ON`, `TEMPO 120`, etc.) with minimal spacing, making quick scanning difficult. | **brief suggested fix:** Group controls into spaced clusters and increase horizontal padding between chips.

3. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Scope/status terms like `ACT TRACK`, `ARMED/ACT`, and `ON` are cryptic and not self-explanatory at a glance. | **brief suggested fix:** Expand labels (or provide a legend/help hint) so state and scope are unambiguous.

4. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards are mostly large empty blocks with little visible metadata, which reads like missing content rather than intentional layout. | **brief suggested fix:** Add key device details (port, channel count, activity) or reduce card height to match actual content.

5. **severity: low** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay has substantial unused space below the table while only rows 1–19/30 are shown, weakening information density. | **brief suggested fix:** Show more rows in available space or tighten overlay height.

6. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Table column spacing is inconsistent; narrow columns (`TYPE`, `DEVICE`) feel cramped while `TARGET` dominates width. | **brief suggested fix:** Rebalance column widths to improve readability of short fields and reduce wasted space.

7. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Heavy multi-color coding (green/blue/orange/pink/cyan) lacks explicit legend and may imply meaning users cannot decode quickly. | **brief suggested fix:** Add a compact color legend or label each section with explicit semantic tags.

8. **severity: low** | **screenshot/page:** `timeline-focused.png` | **issue:** State communication for selected context (`TRACK T1`) is subtle relative to nearby controls (`RESET SONG LOOP`), so focus context can be missed. | **brief suggested fix:** Increase contrast/prominence of the active context chip and keep destructive/reset actions visually secondary.

9. **severity: low** | **screenshot/page:** `mappings-overlay.png` | **issue:** Header action hints (`F5 CLOSE`, `W WRITE`) are terse and easy to miss; interaction model is not obvious for first-time users. | **brief suggested fix:** Use clearer action text (for example, `F5 Close Overlay`, `W Save`) and slightly stronger visual emphasis.

10. **severity: low** | **screenshot/page:** `timeline.png` and `timeline-focused.png` | **issue:** Very small typography in dense track columns reduces legibility and increases cognitive load in the primary working view. | **brief suggested fix:** Increase font size one step for critical labels and reduce non-essential micro-labels in the same area.