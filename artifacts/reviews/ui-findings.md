Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** The top control cluster (transport/mode chips + right-side status chips like `LINK OFF`, `START/STOP OFF`, `QUANT 1/16`) is very dense and visually similar, so state vs action is hard to parse quickly. | **brief suggested fix:** Separate status indicators from actionable controls with distinct grouping and styling (e.g., muted “status” row, stronger button affordance for actions).

2. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Track header content is cramped (`THRU TRACK n`, `TRACK n`, `SONG`, `LOOP`, `DUB`) and requires effort to scan across 12 narrow columns. | **brief suggested fix:** Increase header height/column width or reduce concurrent labels (progressive disclosure, tooltips, or abbreviated labels with legend).

3. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Table rows are tightly packed with minimal vertical separation, making row tracking across `SOURCE -> TARGET -> SCOPE -> ON` error-prone. | **brief suggested fix:** Add subtle zebra striping or larger row height/padding to improve horizontal scan and reduce misread risk.

4. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Top-right metadata (`ROWS 1 / 30`, `ON`) has weak hierarchy and can be mistaken for table content rather than page state. | **brief suggested fix:** Move summary stats into a dedicated status bar block with a label (e.g., “List status”) and stronger contrast.

5. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are easy to miss and don’t clearly communicate primary/secondary action priority. | **brief suggested fix:** Promote primary action visually (button-like treatment) and place help shortcuts in a distinct, lower-emphasis area.

6. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device card states (`DEF`, `SEL`) are encoded as tiny corner tags with limited prominence, making default vs selected status easy to miss. | **brief suggested fix:** Use larger, explicit state badges and/or row-level highlighting with a legend for state semantics.

7. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Right-side row controls (`+`, `SELECT`, `TAP +/-`, `TOGGLE`) are inconsistent in wording and structure, so interaction model is unclear. | **brief suggested fix:** Standardize control patterns per row type (same button order/labels) and add short inline helper text for special controls like `TAP +/-`.

8. **severity: low** | **screenshot/page:** `routing.png` | **issue:** The `ACTIVE 1`/`THRU OFF` header chips and `TRACK 1` context line feel loosely related, weakening hierarchy of “what is being edited now.” | **brief suggested fix:** Consolidate current-edit context into one prominent header block (track, mode, thru state) with consistent visual grouping.