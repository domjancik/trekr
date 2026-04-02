Findings

1. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Track header controls and status tokens are visually overcrowded (`THRU TRACK`, loop step chips, per-track mode labels), making it hard to parse what belongs to which track at a glance. | **brief suggested fix:** Increase horizontal padding and split header content into 2 rows (identity row + controls row) or collapse secondary tokens behind a focused state.

2. **severity: high** | **screenshot/page:** `timeline-focused.png` | **issue:** Right-edge micro-label cluster in the focused track header (transpose/effect state tokens) is cramped and partially ambiguous due to tiny spacing. | **brief suggested fix:** Group these into a dedicated “Track State” strip with separators and minimum token width; hide non-critical chips until focus/hover.

3. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Repeated `SET`/`TGL` buttons are visually similar and close to adjacent value fields, so control intent is unclear (set value vs toggle enable). | **brief suggested fix:** Differentiate button styles by action type (e.g., filled for toggles, outlined for set/apply) and add short labels like `SET KIND`, `TOGGLE ON`.

4. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Section hierarchy is weak between `REC/MON`, `INPUT FX`, and `OUTPUT FX`; all panels share near-identical visual weight. | **brief suggested fix:** Strengthen hierarchy with clearer section titles, slightly larger heading text, and more distinct panel spacing/dividers.

5. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Large empty device list regions look like inactive placeholders, but state messaging is missing (no count, no hint, no selection instructions). | **brief suggested fix:** Add explicit empty/selection helper text and device counts (`1 input detected`, `Select default output`) inside panels.

6. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Row density is very high; columns (`TYPE/DEVICE/SOURCE/TARGET/SCOPE`) have tight spacing, reducing scan speed and increasing misread risk. | **brief suggested fix:** Increase row height slightly, add subtle zebra striping or stronger column separators, and prioritize primary columns with wider widths.

7. **severity: low** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay header metadata (`ROWS 1-19/30`, `SCOPE`) is detached from table headers, so context mapping is not immediate. | **brief suggested fix:** Align metadata directly with the corresponding column header row or move it into a compact top-right status bar with clearer labels.

8. **severity: low** | **screenshot/page:** `mappings.png` and `mappings-overlay.png` | **issue:** Bottom shortcut/footer bar has many similarly styled key hints, making primary actions hard to identify quickly. | **brief suggested fix:** Emphasize top 2–3 primary actions with stronger contrast and demote secondary shortcuts with lower-intensity text.

9. **severity: low** | **screenshot/page:** `timeline.png` | **issue:** Top utility controls (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`) are tightly packed at the right edge, weakening readability and state clarity. | **brief suggested fix:** Add consistent spacing groups and a dedicated transport-state cluster with clearer active/inactive styling.

10. **severity: low** | **screenshot/page:** all pages | **issue:** Active-state communication relies heavily on subtle color shifts; in dense areas this is easy to miss quickly. | **brief suggested fix:** Pair color with secondary cues (icon, marker, underline, or prefix text like `ON`/`ACTIVE`) for faster state recognition.