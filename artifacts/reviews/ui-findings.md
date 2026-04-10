Findings

1. **Severity: medium | Screenshot: `timeline.png` | Issue:** Top control bars are very dense (`PLAY OFF`, `RECORD OFF`, `MODE OVERDUB`, `WRAP EXTEND`, etc.) with minimal grouping, so scanability is low. **Suggested fix:** Add stronger visual grouping (section headers/background bands) and increase horizontal padding between control clusters.

2. **Severity: high | Screenshot: `timeline.png` | Issue:** Track column headers and mini-controls (e.g., `ARM/REC/MUT/SOL`, small numeric chips, `+2`) are cramped and borderline clipped at this resolution. **Suggested fix:** Increase min width for each track card or reduce concurrent columns shown before switching to horizontal scroll/paging.

3. **Severity: medium | Screenshot: `timeline-focused.png` | Issue:** Focused state is not clearly distinct from non-focused layouts beyond width change; hierarchy between `SONG` and `LOOP` panes is still subtle. **Suggested fix:** Add stronger focused-track affordance (accent border/title badge) and clearer pane titles with higher contrast.

4. **Severity: low | Screenshot: `mappings.png` | Issue:** The row-level action hints in the footer are cryptic (`W WRITE`, `F8 DIRECT`, `N NEW`) and rely on memorized hotkeys. **Suggested fix:** Use clearer labels (`Write`, `Direct Map`, `New`) with hotkeys as secondary text.

5. **Severity: medium | Screenshot: `mappings.png` | Issue:** Scope/status columns (`GLOBAL`, `ACT TRACK`, `ON`) visually merge with adjacent fields, making row parsing harder. **Suggested fix:** Increase column separation and apply distinct background tint or vertical separators for status columns.

6. **Severity: medium | Screenshot: `mappings-overlay.png` | Issue:** Overlay title and command row (`F5 CLOSE`, `W WRITE`) are separated from table headers with weak hierarchy; first-time users may miss available actions. **Suggested fix:** Promote command row into a dedicated toolbar strip with stronger contrast and spacing.

7. **Severity: low | Screenshot: `midi-io.png` | Issue:** Large empty device panels dominate the page, while actionable controls are sparse and visually ambiguous (`DEF`, `SEL` tags). **Suggested fix:** Add explicit empty-state/help text and convert tiny tags into clear buttons (`Set Default`, `Select`).

8. **Severity: medium | Screenshot: `routing.png` | Issue:** Similar visual treatment for editable fields, toggles, and buttons (`SET`, `TGL`, value cells) makes control intent unclear. **Suggested fix:** Differentiate control types with consistent shapes/colors (e.g., button fill vs input fill vs toggle state color).

9. **Severity: medium | Screenshot: `routing.png` | Issue:** Multi-panel density (Signal, Rec/Mon, Input FX, Output FX) creates weak reading order; users may not know where to start. **Suggested fix:** Number or sequence sections, or add stronger section headers with short purpose labels and more vertical spacing.

10. **Severity: low | Screenshot: `mappings.png` and `routing.png` | Issue:** Top navigation tab active state is subtle and easy to miss during fast context switches. **Suggested fix:** Increase active-tab contrast and add an additional indicator (underline/icon/state chip) for clearer page state communication.