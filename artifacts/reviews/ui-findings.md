Findings

1. **severity: high** — **screenshot/page:** `timeline.png` — **issue:** The top control strip is visually dense and several grouped toggles (`PLAY OFF`, `RECORD OFF`, `MODE OVERDUB`, etc.) read like one long run-on label block, making state scanning slow. — **brief suggested fix:** Increase horizontal padding between chips/groups and add stronger group separators or section labels so transport/state controls parse at a glance.

2. **severity: high** — **screenshot/page:** `timeline-focused.png` — **issue:** The focused mode expands content but keeps many controls at very small text size; hierarchy between global controls vs track-local controls is weak, so users can’t quickly tell what affects track vs song. — **brief suggested fix:** Elevate section headers and apply distinct styling for global vs track-local control bands (contrast, spacing, and clearer labels).

3. **severity: medium** — **screenshot/page:** `mappings.png` — **issue:** Row content is cramped; `SOURCE`, `TARGET`, `SCOPE`, and `ON` columns have minimal breathing room and repeated `ON` labels look like static text rather than toggles. — **brief suggested fix:** Increase row height/padding and style active toggles as explicit controls (button/checkbox affordance) instead of plain text chips.

4. **severity: medium** — **screenshot/page:** `mappings.png` — **issue:** The top status controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) have equal visual weight to editable fields, so it’s unclear which are mode indicators vs actionable controls. — **brief suggested fix:** Differentiate status pills from interactive buttons with distinct fill/border treatment and consistent verb-based labels for actionable items.

5. **severity: medium** — **screenshot/page:** `mappings-overlay.png` — **issue:** Overlay header/actions (`F5 CLOSE`, `W WRITE`) are small and low-emphasis relative to table content; discoverability of “how to exit” and “how to commit” is weak. — **brief suggested fix:** Promote overlay actions into a clearer top action bar with stronger contrast and explicit verbs (e.g., `Close (F5)`, `Save (W)`).

6. **severity: medium** — **screenshot/page:** `midi-io.png` — **issue:** Inputs and outputs panes use very large empty card bodies with minimal device metadata, making the page feel sparse and ambiguous when few devices are present. — **brief suggested fix:** Use compact rows with status badges (default/selected/connected) and reserve large panel space only when details are expanded.

7. **severity: low** — **screenshot/page:** `midi-io.png` — **issue:** `DEF` and `SEL` tags are tiny and tightly placed at the card edge, which risks clipping/legibility issues on smaller scales. — **brief suggested fix:** Add inner padding and minimum tag width; consider icon+label badges with clearer contrast.

8. **severity: high** — **screenshot/page:** `routing.png` — **issue:** Mixed color coding (green/blue/orange/pink) across signal, record/monitor, input FX, output FX lacks a visible legend, so meaning of colors is not self-evident. — **brief suggested fix:** Add a persistent legend or consistent semantic color mapping with section labels that explicitly state what each color represents.

9. **severity: medium** — **screenshot/page:** `routing.png` — **issue:** Many `SET`, `TGL`, `+`, `-` controls are visually similar and repeated, making intended action unclear (select vs increment vs toggle). — **brief suggested fix:** Standardize control types with distinct shapes/labels (e.g., toggle switch, stepper, action button) and add concise inline tooltips/help text.

10. **severity: low** — **screenshot/page:** `mappings.png`, `mappings-overlay.png`, `routing.png`, `timeline.png` — **issue:** Bottom shortcut strip (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) is persistent but low hierarchy and can be mistaken for passive status text. — **brief suggested fix:** Promote as a dedicated command bar with clearer button affordance and active-page highlighting.