Findings

1. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** The top track-mode chips and right-side transport/status cluster compete visually with similar weight, so primary context (what mode am I in?) is not immediately obvious. | **brief suggested fix:** Make one clear primary status band (mode + active scope) and demote secondary stats with lower contrast or smaller type.

2. **severity: high** | **screenshot/page:** `timeline-focused.png` | **issue:** Dense symbols (`↑↓x`, small channel/loop markers, compact numeric lanes) are hard to parse at a glance and feel cryptic without immediate legend cues. | **brief suggested fix:** Expand spacing around micro-controls and add short inline labels/tooltips for icon-only affordances.

3. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Row content is heavily compressed; `TYPE/DEVICE/SOURCE/TARGET/SCOPE/ON` columns visually merge, making scanning and comparison slow. | **brief suggested fix:** Increase horizontal padding and strengthen column separation (subtle alternating column tint or clearer dividers).

4. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command strip (`F5 CLOSE`, `W WRITE`) is easy to miss and not strongly differentiated from table headers, weakening modal-state communication. | **brief suggested fix:** Add a distinct modal header bar/background and clearer “overlay active” treatment.

5. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Large empty card bodies dominate while critical state (`DEF`, `SEL`) is tiny and tucked into corners, so selection/default status is easy to overlook. | **brief suggested fix:** Promote selected/default badges with larger, persistent labels and reduce empty-body visual weight.

6. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Mixed color semantics (green, blue, orange, pink, gray) are reused for different meanings across blocks, which can mislead users about what color represents. | **brief suggested fix:** Define a strict color meaning system (e.g., input/output/armed/disabled) and apply consistently across panels.

7. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Button labels like `SET`, `TGL`, and `+` are terse/ambiguous in repeated contexts; intent is not always clear without prior knowledge. | **brief suggested fix:** Use slightly more explicit labels (`Set Device`, `Toggle`, `Add`) or add contextual suffixes.

8. **severity: low** | **screenshot/page:** `mappings.png`, `mappings-overlay.png` | **issue:** Some top-right counters (`ROWS 1 / 30`, `ROWS 1-19 / 30`) are visually detached from main list headers, reducing hierarchy clarity. | **brief suggested fix:** Attach counters directly to table header row and align baseline with column labels.