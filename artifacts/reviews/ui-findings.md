Findings

1. severity: medium; screenshot/page name: `mappings`; issue: The selected-row highlight, dense table grid, and all-caps labels compete visually, so the primary state and editable focus are hard to parse quickly. brief suggested fix: Increase contrast between selected row, focused cell, and default rows, and reduce nonessential line weight or brightness in the rest of the table.

2. severity: medium; screenshot/page name: `mappings`; issue: The footer command strip is cramped and cryptic (`W WRITE`, `N NEW`, `DEL REMOVE`, `Q/E ADJUST`), which makes the available actions hard to understand at a glance. brief suggested fix: Add clearer verb labels or group commands with more spacing and stronger separation between action clusters.

3. severity: low; screenshot/page name: `mappings`; issue: `ROWS 1 / 26` and `SCOPE ON` are visually detached from the main controls and read like status fragments rather than clear state indicators. brief suggested fix: Group row count and filter/state info into a labeled status area near the table header.

4. severity: medium; screenshot/page name: `mappings-overlay`; issue: The overlay looks visually disconnected from the underlying app state and the header line (`F5 CLOSE   W WRITE`) is easy to miss, so it is not immediately obvious whether this is read-only help, an editor, or a modal selection view. brief suggested fix: Add a stronger modal title/subtitle and a clearer status/action bar that explicitly states what the overlay is for.

5. severity: low; screenshot/page name: `mappings-overlay`; issue: The large empty lower area creates weak hierarchy and makes the overlay feel unfinished relative to the dense table content above. brief suggested fix: Either tighten the modal height to content or use the lower area for pagination/help text aligned to the table.

6. severity: medium; screenshot/page name: `midi-io`; issue: The large empty device panels dominate the page, making it look like content failed to load even though devices are present. brief suggested fix: Reduce empty panel height or add explicit empty-state/list framing so the visible device cards feel intentional.

7. severity: medium; screenshot/page name: `midi-io`; issue: `DEF` and `SEL` badges are tiny and ambiguous, especially when stacked into the device card corner with little spacing. brief suggested fix: Expand them into clearer labels or chips such as `Default` and `Selected`, with stronger visual distinction.

8. severity: low; screenshot/page name: `routing`; issue: The top control cluster mixes track selector, thru state, description text, and `TAP VALUE` without a clear reading order. brief suggested fix: Separate mode/state controls from explanatory text and align the primary editable control more prominently.

9. severity: medium; screenshot/page name: `routing`; issue: The right-edge action buttons (`TAP +/-`, `SELECT`, `TOGGLE`) look similar despite doing different things, so affordance is weak and scanning each row takes extra effort. brief suggested fix: Differentiate action types by label clarity, width, and/or color treatment, and align them consistently as a dedicated action column.

10. severity: low; screenshot/page name: `routing`; issue: The left label column and the large value fields have inconsistent visual balance, with a lot of unused horizontal space inside the value areas. brief suggested fix: Rebalance column widths so labels, values, and actions feel more intentionally proportioned.

11. severity: medium; screenshot/page name: `timeline`; issue: The top toolbar is extremely dense, with many equal-weight toggles (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, etc.), which weakens hierarchy and makes important transport state harder to identify. brief suggested fix: Group transport, timing, and mode controls into separate clusters and give active playback/record states stronger prominence.

12. severity: low; screenshot/page name: `timeline`; issue: `F6 LINK` is visually isolated on the far right and can be missed as a relevant control/state. brief suggested fix: Move it into the main transport/settings cluster or style it as a clearly associated shortcut hint.

13. severity: medium; screenshot/page name: `timeline`; issue: Track headers and loop headers repeat in a way that is initially hard to decode, especially with the narrow columns and similar visual treatment across all lanes. brief suggested fix: Strengthen the distinction between track and loop columns with clearer header grouping and slightly more breathing room in header labels.

14. severity: low; screenshot/page name: `all`; issue: Overall spacing is internally consistent at a grid level, but many labels and controls are packed tightly enough that the UI feels harder to scan than necessary. brief suggested fix: Add small increases in padding around headers, status chips, and footer/control bars to improve first-pass readability without changing the visual style.