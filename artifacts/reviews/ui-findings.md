Findings

1. severity: high, screenshot/page name: `timeline`, issue: The top control bar is overloaded with many same-sized status pills (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, `TEMP 120`, `0 1/16`, etc.), so primary transport state and editable controls do not stand out and the page is hard to parse quickly. brief suggested fix: Group controls by function, increase contrast for the primary transport/state indicators, and visually separate readouts from actionable buttons.

2. severity: high, screenshot/page name: `midi-io`, issue: The large device rows look like meters or empty progress bars rather than selectable inputs/outputs, and the small `DEF SEL` badge is too cryptic to explain selected/default state. brief suggested fix: Add explicit selection markers and plain-language badges like `Selected` and `Default`, and reduce the bar-like fill treatment so rows read as list items.

3. severity: medium, screenshot/page name: `mappings`, issue: The table is very dense and the right-side columns (`SCOPE`, `ON`) are cramped, making row meaning and state harder to scan. brief suggested fix: Widen or rebalance the state columns, or reduce nonessential width in `TARGET` so state fields remain legible at a glance.

4. severity: medium, screenshot/page name: `mappings`, issue: The footer shortcut legend is crowded and uses abbreviated labels like `W WRITE`, `N NEW`, and `DEL REMOVE`, which is easy to misread, especially while the page is in `TAP MODE: READ ONLY`. brief suggested fix: Separate mode/status from available actions, and use clearer labels such as `Press W to Write` only when that action is actually available.

5. severity: medium, screenshot/page name: `mappings-overlay`, issue: The overlay shows `ROWS 1-19 / 26` but there is no strong visual cue that the list is truncated or scrollable, so users may assume they are seeing the full mapping set. brief suggested fix: Add an explicit scrollbar, paging affordance, or a stronger `more rows below` indicator.

6. severity: medium, screenshot/page name: `routing`, issue: The header area duplicates information (`ACTIVE TRACK ROUTING`, `TRACK 1`, and the explanatory sentence) while the `TAP VALUE` block floats separately on the right, weakening hierarchy and making the main state harder to understand. brief suggested fix: Consolidate the track/context information into one clear header block and align `TAP VALUE` with the editable control it affects.

7. severity: medium, screenshot/page name: `routing`, issue: The row-end action labels (`TAP +/-`, `SELECT`, `TOGGLE`) are small, low-emphasis, and visually detached from the values they modify. brief suggested fix: Turn them into clearer button-like controls with stronger contrast and tighter association to the active field.

8. severity: low, screenshot/page name: `timeline`, issue: The segmented options under the page title (`TIMELINE`, `VERTICAL`, `SONG COLUMNS + LOOP DETAIL`) do not clearly communicate whether they are tabs, filters, or current view state. brief suggested fix: Use a more explicit segmented-control treatment with a stronger active state and clearer grouping label.

9. severity: low, screenshot/page name: `mappings-overlay`, issue: There is a large amount of empty space below the visible rows, which makes the overlay feel visually unbalanced and understates the importance of the content. brief suggested fix: Reduce overlay height or increase visible row count so the content fills the panel more naturally.

10. severity: low, screenshot/page name: `midi-io`, issue: Spacing is inconsistent between the dense device lists and the large empty area at the bottom of the page, which makes the layout feel unfinished. brief suggested fix: Either tighten the overall page height or use the lower area for help text, port details, or state explanation.