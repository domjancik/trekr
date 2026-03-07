Findings

1. **severity: medium | screenshot/page: `timeline.png` | issue:** Track columns are very dense; labels like `ARM / REC / MUT / SOL`, loop tokens, and track names compete in the same narrow header space, making scan-time high. **Suggested fix:** Increase header row height and separate status chips from track title/loop fields (or reduce visible columns at once with horizontal paging).

2. **severity: medium | screenshot/page: `timeline.png` | issue:** Global controls in the top-right (`LINK OFF`, `START/STOP OFF / SHIFT+F6`, `QUANT 1/16`, `PEERS 0`) are tightly packed with weak visual grouping, so it is unclear which controls are related. **Suggested fix:** Group related controls into boxed clusters with spacing and consistent chip widths.

3. **severity: low | screenshot/page: `timeline-focused.png` | issue:** Mode tabs (`TIMELINE`, `VERTICAL`, `FOCUSED TRACK + LOOP DETAIL`) have subtle state contrast; active vs inactive is not immediately obvious. **Suggested fix:** Strengthen selected-state styling (higher contrast fill + clearer indicator icon or underline).

4. **severity: high | screenshot/page: `mappings.png` | issue:** The mappings table is overloaded: many rows plus dense bottom shortcut legend creates cognitive overload, and key hints are hard to parse quickly. **Suggested fix:** Reduce simultaneous on-screen shortcut text, add spacing between logical shortcut groups, and use stronger typographic hierarchy for primary actions.

5. **severity: medium | screenshot/page: `mappings-overlay.png` | issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are small and easy to miss; scope/count metadata on the right is visually detached from table context. **Suggested fix:** Promote primary overlay actions into a clearer top action bar and align metadata with column headers.

6. **severity: medium | screenshot/page: `midi-io.png` | issue:** Device cards contain large unlabeled gray interior regions that look like empty placeholders; unclear whether they represent activity, channels, or just card body. **Suggested fix:** Add explicit labels or remove the inner fill panel; if it represents signal/status, add a legend or meter semantics.

7. **severity: medium | screenshot/page: `routing.png` | issue:** Right-edge row actions are inconsistent (`TAP +/-` on input device vs `SELECT` elsewhere, `TOGGLE` on passthrough), which weakens predictability. **Suggested fix:** Standardize action slot behavior and wording, then expose row-specific action as secondary text/icon.

8. **severity: low | screenshot/page: `midi-io.png` and `routing.png` | issue:** Long device names are close to control boundaries and may clip/truncate unpredictably with slightly longer strings. **Suggested fix:** Reserve fixed action-column width and apply explicit truncation with ellipsis + tooltip/full-name reveal.