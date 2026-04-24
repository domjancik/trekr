Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Information density is very high (tiny text, many compact control chips, dense track columns), so primary state and next action are hard to parse quickly. | **brief suggested fix:** Increase font size/line height for top control bars and add stronger grouping (clear section headers + more vertical spacing between global controls and track content).

2. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Some labels are hard to read or look compressed/ambiguous (e.g., `NOTEADD OFF`, `QUANT 1/16`, `PEERS 0` among many adjacent chips). | **brief suggested fix:** Add minimum chip width/padding and reduce chip count per row by wrapping into two rows or collapsing secondary controls.

3. **severity: low** | **screenshot/page:** `timeline-focused.png` | **issue:** The focused layout improves clarity, but top-right status/action text (`F6 / SHIFT+F6`, launch/sync chips) remains visually weak and easy to miss. | **brief suggested fix:** Promote transport/sync states with stronger contrast and larger type, and separate keyboard hints from live state.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Table columns are crowded and state chips (`GLOBAL`, `ACT TRACK`, `ARMED/ACT`) are visually similar, making scope/state scanning slow. | **brief suggested fix:** Differentiate scope/state with distinct color semantics and add slightly wider spacing between `TARGET`, `SCOPE`, and `ON` columns.

5. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Bottom shortcut legend is very small and low hierarchy compared with the dense grid above, so discoverability is weak. | **brief suggested fix:** Increase legend size/contrast or move key actions to a clearer, separated help strip.

6. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay header and table structure are cleaner, but pagination/state (`ROWS 1-9 / 30`) is easy to overlook and not strongly tied to navigation controls. | **brief suggested fix:** Pair row range with explicit nav affordances (e.g., `PgUp/PgDn`) in the same visual block and increase contrast.

7. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Large gray card bodies dominate visual weight while actionable state is tiny (`DEF`, `SEL`), making it unclear what is selected vs merely listed. | **brief suggested fix:** Reduce empty-body prominence and amplify selected/default indicators with clearer badges and stronger border/state treatment.

8. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Repeated compact controls (`+`, `SET`, `TGL`) are unclear in meaning without context; affordances look similar despite different effects. | **brief suggested fix:** Replace generic micro-labels with explicit verbs (`Increment`, `Apply`, `Toggle`) or add concise inline tooltips/help text.

9. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Section hierarchy is decent, but intra-panel spacing is inconsistent (tight label rows vs larger control blocks), creating uneven rhythm and slower scanning. | **brief suggested fix:** Standardize vertical spacing tokens for label rows, control rows, and panel gutters to create predictable scan patterns.