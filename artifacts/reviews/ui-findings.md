Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Column-mode track headers and control labels are heavily truncated (`S...`, `V...`, `+ ADD OUTPUT FX` crowded), making per-track state hard to parse quickly. | **brief suggested fix:** Increase minimum header/footer label width or show full text on focus/hover with consistent abbreviations and tooltips.

2. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** Top control/status strip is dense with many equal-weight toggles (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`) and weak grouping, so active transport/launch state is easy to miss. | **brief suggested fix:** Group controls into labeled clusters (Transport, Launch, Quantize), and increase contrast/weight for active states.

3. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Repeated micro-controls (`+`, `SET`, `TGL`) are visually similar and ambiguous about immediate vs staged actions. | **brief suggested fix:** Differentiate button styles by action type (toggle vs commit), and add short inline labels or icons for intent.

4. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Vertical spacing between blocks is inconsistent (e.g., FX rows vs signal rows), which creates visual noise in an already dense screen. | **brief suggested fix:** Normalize row heights/padding and align section baselines across left/right panes.

5. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** `DEF` and `SEL` badges are very small and placed tightly at row edges, reducing scanability of default/selected device state. | **brief suggested fix:** Increase badge size/contrast and separate status chips from row content with fixed spacing.

6. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Rightmost columns (`SCOPE`, `ON`) are narrow and crowded; values like `ARMED/ACT` and `ACT TRACK` compete for space and readability. | **brief suggested fix:** Rebalance table column widths (slightly reduce `TARGET`, expand `SCOPE`/state columns) and enforce consistent text truncation rules.

7. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay header hierarchy is weak: primary title, shortcut hints, row count, and scope metadata have similar weight and scattered alignment. | **brief suggested fix:** Use a 2-row header structure (title/actions first, metadata second) with clearer alignment and stronger typographic contrast.

8. **severity: low** | **screenshot/page:** `mappings.png`, `mappings-overlay.png` | **issue:** Key hints at the bottom (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) look like passive labels rather than interactive shortcuts. | **brief suggested fix:** Style shortcut chips as explicit actions (button affordance + active/inactive state color) and add one-line contextual hint near the active pane.