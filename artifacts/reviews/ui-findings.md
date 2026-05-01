Findings

1. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** The top mode bar (`PLAY/REC/...`) and per-track control rows are very dense with near-equal visual weight, so primary state is hard to scan quickly. | **brief suggested fix:** Increase contrast/size for global transport state, reduce track-row chrome, and add clearer grouping separators between global controls and per-track controls.

2. **severity: high** | **screenshot/page:** `timeline-focused.png` | **issue:** The focused-track view still presents many small labels (`REC`, `MUT`, `SOL`, loop markers, tiny badges) in compressed horizontal bands, making active state ambiguous at a glance. | **brief suggested fix:** Promote active statuses with stronger filled states and reserve muted styling for inactive labels; add more vertical spacing between header bands.

3. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Table columns are visually crowded; `SOURCE`, `TARGET`, `SCOPE`, and trailing `ON` controls look tightly packed, and the rightmost toggles feel clipped into narrow cells. | **brief suggested fix:** Widen/right-pad the final columns and reduce nonessential row border weight to improve readability of row actions.

4. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay title/actions (`F5 CLOSE`, `W WRITE`) and metadata (`ROWS 1-19 / 30`, `SCOPE`) are small and weakly separated from table content, so overlay context is easy to miss. | **brief suggested fix:** Add a stronger header bar (background contrast + spacing) and align metadata/actions into clearer left/right blocks.

5. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Large empty card interiors dominate inputs/outputs, while actionable states (`DEF`, `SEL`) are tiny tags in corners, making selection status unclear. | **brief suggested fix:** Increase prominence of selected/default state (stronger badge size/placement) and reduce unused empty area with clearer per-device summary rows.

6. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Many controls share similar visual treatment (`SET`, `TGL`, small +/- boxes), so control intent (toggle vs picker vs action) is not immediately distinguishable. | **brief suggested fix:** Differentiate control types with distinct shapes/colors and short inline labels/icons for behavior cues.

7. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Section spacing is inconsistent between left signal panel and right FX/rec-monitor blocks, creating slight alignment drift and weaker scan rhythm. | **brief suggested fix:** Normalize vertical spacing and baseline alignment across sibling panels.

8. **severity: low** | **screenshot/page:** `mappings.png`, `routing.png`, `timeline.png` | **issue:** Bottom helper/action strips (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) are visually understated relative to dense content, so navigation affordance can be missed. | **brief suggested fix:** Raise contrast and spacing for footer actions, and keep their placement/weight consistent across pages.